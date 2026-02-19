use async_process::Command;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::collections::HashSet;

use super::{Job, JobState};

/// Options for querying recent-ended jobs from Slurm accounting (`sacct`).
#[derive(Debug, Clone)]
pub struct SacctOptions {
    /// Limit to this user (recommended to match the default `squeue` behavior).
    pub user: Option<String>,
    /// Terminal states to include.
    pub states: Vec<JobState>,
    /// Limit to these partitions (optional).
    pub partitions: Vec<String>,
    /// Limit to these QoS values (optional).
    pub qos: Vec<String>,
    /// Look back window in hours.
    pub recent_hours: u32,
    /// Which sacct fields to request, in order.
    pub format_fields: Vec<&'static str>,
}

impl SacctOptions {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Output format and shape.
        args.push("-n".to_string()); // no header
        args.push("-P".to_string()); // parsable2, '|' delimited
        args.push("-X".to_string()); // allocations only (avoid job steps)

        // Time window.
        // We want jobs that were in the selected states during the window.
        // We'll still optionally filter by end-time in-app if needed later.
        args.push("-S".to_string());
        args.push(format!("now-{}hours", self.recent_hours.max(1)));
        args.push("-E".to_string());
        args.push("now".to_string());

        // Filters.
        if let Some(user) = &self.user {
            if !user.is_empty() {
                args.push("--user".to_string());
                args.push(user.clone());
            }
        }

        if !self.partitions.is_empty() {
            args.push("--partition".to_string());
            args.push(self.partitions.join(","));
        }

        if !self.qos.is_empty() {
            args.push("--qos".to_string());
            args.push(self.qos.join(","));
        }

        if !self.states.is_empty() {
            let states = self
                .states
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(",");
            args.push("--state".to_string());
            args.push(states);
        }

        // Format fields.
        let mut unique = HashSet::new();
        let mut fields = Vec::new();
        for f in &self.format_fields {
            if unique.insert(*f) {
                fields.push(*f);
            }
        }
        if fields.is_empty() {
            // Keep this explicit to avoid surprising default output shapes.
            fields = vec!["JobIDRaw", "JobName", "User", "State", "Elapsed", "NodeList", "AllocCPUS"];
        }

        args.push("--format".to_string());
        args.push(fields.join(","));

        args
    }
}

/// Run `sacct` and parse its output into `Job` rows.
pub async fn run_sacct(options: &SacctOptions) -> Result<Vec<Job>> {
    let args = options.to_args();
    let output = Command::new("sacct").args(&args).output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("sacct failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    parse_sacct_output(&stdout, &options.format_fields)
}

fn parse_sacct_output(stdout: &str, format_fields: &[&'static str]) -> Result<Vec<Job>> {
    if stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Ensure we dedupe fields exactly as `to_args()` does, so indexes match.
    let mut unique = HashSet::new();
    let mut fields = Vec::new();
    for f in format_fields {
        if unique.insert(*f) {
            fields.push(*f);
        }
    }
    if fields.is_empty() {
        fields = vec!["JobIDRaw", "JobName", "User", "State", "Elapsed", "NodeList", "AllocCPUS"];
    }

    let mut jobs = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.is_empty() {
            continue;
        }

        let mut job = Job::default();

        for (idx, raw_value) in parts.iter().enumerate() {
            if idx >= fields.len() {
                break;
            }
            let value = raw_value.trim();
            if value.is_empty() || value == "Unknown" || value == "N/A" {
                continue;
            }

            match fields[idx] {
                "JobIDRaw" | "JobID" => job.id = value.to_string(),
                "JobName" => job.name = value.to_string(),
                "User" => job.user = value.to_string(),
                "State" => job.state = value.parse().unwrap_or(JobState::Other),
                "Elapsed" => job.time = value.to_string(),
                "NNodes" => job.nodes = value.parse::<u32>().unwrap_or(0),
                "NodeList" => job.node = Some(value.to_string()),
                "AllocCPUS" | "NCPUS" => job.cpus = value.parse::<u32>().unwrap_or(0),
                "ReqMem" => job.memory = value.to_string(),
                "Partition" => job.partition = value.to_string(),
                "QOS" => job.qos = value.to_string(),
                "Account" => job.account = Some(value.to_string()),
                "Priority" => job.priority = value.parse::<u32>().ok(),
                "WorkDir" => job.work_dir = Some(value.to_string()),
                "Submit" => job.submit_time = Some(value.to_string()),
                "Start" => job.start_time = Some(value.to_string()),
                "End" => job.end_time = Some(value.to_string()),
                "Reason" => job.pending_reason = Some(value.to_string()),
                _ => {}
            }
        }

        if job.id.is_empty() {
            continue;
        }

        jobs.push(job);
    }

    Ok(jobs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sacct_output_basic() {
        let stdout = "123|myjob|alice|COMPLETED|00:10:00|2|node[1-2]|16|2048Mc|part|normal|proj|1000|/tmp|2026-01-01T00:00:00|2026-01-01T00:00:01|2026-01-01T00:10:01|None\n";
        let fields = vec![
            "JobIDRaw",
            "JobName",
            "User",
            "State",
            "Elapsed",
            "NNodes",
            "NodeList",
            "AllocCPUS",
            "ReqMem",
            "Partition",
            "QOS",
            "Account",
            "Priority",
            "WorkDir",
            "Submit",
            "Start",
            "End",
            "Reason",
        ];
        let jobs = parse_sacct_output(stdout, &fields).unwrap();
        assert_eq!(jobs.len(), 1);
        let j = &jobs[0];
        assert_eq!(j.id, "123");
        assert_eq!(j.name, "myjob");
        assert_eq!(j.user, "alice");
        assert_eq!(j.state, JobState::Completed);
        assert_eq!(j.time, "00:10:00");
        assert_eq!(j.nodes, 2);
        assert_eq!(j.node.as_deref(), Some("node[1-2]"));
        assert_eq!(j.cpus, 16);
        assert_eq!(j.memory, "2048Mc");
        assert_eq!(j.partition, "part");
        assert_eq!(j.qos, "normal");
    }

    #[test]
    fn parse_sacct_output_skips_empty_lines() {
        let stdout = "\n\n";
        let jobs = parse_sacct_output(stdout, &["JobIDRaw"]).unwrap();
        assert!(jobs.is_empty());
    }
}
