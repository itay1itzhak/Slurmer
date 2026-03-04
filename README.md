# slurmer

A TUI application for monitoring and managing SLURM jobs.

It provides an intuitive, interactive interface to view, filter, sort, and manage SLURM jobs, making SLURM job management more efficient and user-friendly.

## ✨ Features

- **🔄 Real-time Job Monitoring**: View and refresh SLURM job statuses in real-time
  ![](./images/monitor.png)
- **🔍 Advanced Filtering**: Filter jobs by user, state, partition, QoS, job name, and more in real-time(regex supported)
  ![](./images/filter.png)
- **📊 Customizable Columns**: Flexibly configure which job information columns to display and in what order
  ![](./images/columns.png)
- **📝 Job Details View**: Examine job scripts and job logs
  ![](./images/script.png)<br>![](./images/log.png)
- **🎮 Job Management**: Cancel selected jobs
  ![](./images/cancel.png)

<!-- | 🔄 **Real-time Job Monitoring** | 🔍 **Advanced Filtering** | 📊 **Customizable Columns** |
|----------------------------------|---------------------------|------------------------------|
| **View and refresh SLURM job statuses in real-time**<br>![](./images/monitor.png)       | **Filter jobs by user, state, partition, QoS, job name, and more in real-time (regex supported)**<br>![](./images/filter.png)  | **Flexibly configure which job information columns to display and in what order**<br>![](./images/columns.png)    |

| 📝 **Job Details View**         | 🎮 **Job Management**     |                              |
|----------------------------------|---------------------------|------------------------------|
| **Examine job scripts and job logs**<br>![](./images/script.png)<br>![](./images/log.png) | **Cancel selected jobs directly from the UI**<br>![](./images/cancel.png) |                              | -->

## 🛠️ Installation

```bash
cargo install slurmer
```

or install from the latest source code:

```bash
cargo install --git https://github.com/wjwei-handsome/Slurmer.git
```

## 📖 Usage

Just run `slurmer`.

## ⌨️ Keyboard Shortcuts

- <kbd>↓/↑</kbd>: Move up and down in the job list
- <kbd>Shift + ↓/↑</kbd>: Move job in the log-view/script-view
- <kbd>f</kbd>: Open filter menu
- <kbd>c</kbd>: Open column selection menu
- <kbd>s</kbd>: Open settings (e.g. slurm logs dir)
- <kbd>v</kbd>: View job logs
- <kbd>Enter</kbd>: View job script
- <kbd>Space</kbd>: Select job
- <kbd>a</kbd>: Select all jobs
- <kbd>r</kbd>: Refresh job list
- <kbd>x</kbd>: Cancel selected jobs
- <kbd>Esc</kbd>: Quit application

More detailed keybindings can be found each popup menu.

## 🔗 Dependencies

- slurm utilities (e.g., `squeue`, `scancel`) is required.
- [`bat`](https://github.com/sharkdp/bat) is optional for viewing job scripts.

## ⚙️ Configuration

`slurmer` automatically detects available SLURM partitions and QoS in your system and uses the currently logged-in username as the default filter.

To enable viewing logs for completed jobs (when `scontrol` can no longer return `StdOut/StdErr`), configure a base directory to search:\n+\n+- In-app: press `s` and set **Slurm logs dir** (saved to `~/.config/slurmer/config.toml`)\n+- Or environment: set `SLURMER_SLURM_LOGS_DIR=/path/to/slurm_logs` (overrides config file)

## ✅ Testing

- Unit tests:

```bash
cargo test
```

- Manual smoke test:
  - Run `slurmer` and confirm the job list includes **active jobs** plus **recently-ended jobs** (default: last 24 hours).
  - Press `f` and edit **Ended last (hours)**, apply filters (`Ctrl+a`), and confirm the list updates.
  - If `sacct` is unavailable on the cluster, `slurmer` should still show active jobs and display a short status message.

## 👥 Contributing

Contributions are welcome! Feel free to submit issues or pull requests.

## 📜 License

Copyright (c) wjwei-handsome <weiwenjie@westlake.edu.cn>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
