# Feature dependencies

This is the source of truth for dependency notes in the README, website and
release onboarding. Record requirements for the environment executing the
feature, not just the desktop host. Updated 2026-10-12.

| Feature | Required environment/tools | Bundled by Warpai | Missing dependency / recovery |
| --- | --- | --- | --- |
| Local terminal | Supported desktop OS and an available shell | Desktop terminal | Git, Node and Python are not requirements for ordinary terminal use. |
| Local repository collaboration, worktree management and Git Review | System Git executable accessible to the desktop; repository with valid Git metadata | Desktop does not currently bundle Git | Install Git and restart Warpai so it inherits the updated PATH. A cloned directory alone is insufficient. Missing Git must have explicit guidance instead of `invalid_input`. |
| Local CLI Agent | Installed vendor CLI, its runtime and authentication; Agent communication enabled for participation | Native communication bridge, not vendor CLIs | Install/authenticate the CLI using its own instructions. Do not assume every CLI requires Node. |
| SSH file browsing, remote collaboration and Review | Authenticated system OpenSSH, remote SSH/SFTP support, confirmed shell integration and compatible Companion under the same remote account | Companion includes its private Git runtime | Integrate the shell and install/update Companion. Remote infrastructure requires no separate Git, Python, Node, tmux or socat. |
| Windows WSL workspace | WSL 2 distribution, confirmed integrated guest shell/account, WSL communication enabled and dedicated WSL Companion | Windows package includes the guest installation bundle and its private Git runtime | Account discovery needs shell integration but not Companion. File browsing/communication need a working guest Companion; installation/removal must remain available after failure. |
| Remote/WSL CLI Agent | Vendor CLI, runtime and authentication inside the selected host/distribution/account | Companion communication, not vendor CLIs | Installing a vendor CLI only on Windows does not install it in WSL or SSH. |
| Data usage | Supported provider authentication/API configuration or supported usage CLI | Provider-specific adapters | Report unknown/unavailable usage without submitting a paid model prompt. Antigravity CLI usage needs `agy` >= 1.1.11; other providers retain their own requirements. |

Companion's private Git stays private: it does not change user PATH and does not
make `git` available in an interactive desktop or guest terminal. User-run Git
commands and native terminal worktree creation still need Git in that shell.
Local tools do not satisfy guest requirements, and missing desktop Git must not
disable a healthy SSH/WSL Companion.

Before public documentation updates, confirm package contents against the release
source and link this matrix. Keep desktop and Companion requirements separate;
do not advertise the desktop as having no Git dependency for Git features.
