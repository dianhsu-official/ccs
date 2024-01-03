# Simple Competitive Companion Server

Need [Competitive Companion](https://github.com/jmerle/competitive-companion)

## Execute arguments

| Key              | Sample Value                                 | Description                                                                     |
| ---------------- | -------------------------------------------- | ------------------------------------------------------------------------------- |
| `workspace`      | `/home/dianhsu/algorithm`                    | Generate file location.                                                         |
| `templates`      | `/home/dianhsu/sol.cpp,/home/dianhsu/sol.py` | Template file location, multiple templates path should be seperated by a comma. |
| `port`           | `27121`                                      | Listening port of CCS, default is `27121`.                                      |
| `open_by_vscode` | `true` or `false`                            | Enable open vscode after templates generated.                                   |
| `verbose`        |                                              | Verbose show logs.                                                              |
| `log_file`       | `./test.log` or `stderr` or `stdout`         | File path or `stdout` or `stderr`, it's used for log file                       |
| `short_path`     | `true` or `false`                            | Enable short path directory created                                             |