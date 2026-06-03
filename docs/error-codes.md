# PAO Error Codes

PAO errors are rendered as `CODE: message`. The code is stable within the `v0` line, and the message is written for humans.

Exit code `2` means the command request or local configuration needs user action. Exit code `1` means PAO attempted an operation and it failed.

| Code | Exit | Meaning |
| --- | ---: | --- |
| `PAO-0001` | 2 | CLI arguments are invalid. |
| `PAO-0004` | 2 | The requested command is not implemented. |
| `PAO-1001` | 2 | A PAO workspace already exists. |
| `PAO-1002` | 2 | A PAO workspace is required. |
| `PAO-1003` | 2 | The PAO workspace file is invalid or unsupported. |
| `PAO-1101` | 2 | Repository input is invalid. |
| `PAO-1102` | 2 | The repository is already registered. |
| `PAO-1103` | 2 | The repository is not registered. |
| `PAO-1104` | 2 | The target checkout path already exists. |
| `PAO-1105` | 1 | A git command failed. |
| `PAO-1201` | 2 | The user configuration is invalid. |
| `PAO-1202` | 2 | AI client input is invalid. |
| `PAO-1203` | 2 | An AI client is required. |
| `PAO-1301` | 2 | Task input is invalid. |
| `PAO-1302` | 2 | The task already exists. |
| `PAO-1303` | 2 | The task does not exist. |
| `PAO-9001` | 1 | A filesystem or process I/O operation failed. |
| `PAO-9002` | 1 | A YAML or JSON serialization operation failed. |
