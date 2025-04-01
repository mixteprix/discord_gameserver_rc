This directory contains scripts for controlling the gameservers.

The example_server directory should be removed before deployment.

Folder and control scripts must adhere to the following structure.
```
gameservers
├── [SERVER_NAME]
│   ├── config.json
│   ├── start.sh
│   ├── status.sh
│   └── stop.sh
...
```

The use of the screen program as in the example is optional, but recommended to allow easier attachment to the running gameservers.