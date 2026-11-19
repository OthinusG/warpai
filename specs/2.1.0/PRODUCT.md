# Guarded process panel

Show Warpai Processes in Agent Communication when the focused SSH or WSL
terminal confirms a running guardian for its remote account. The subpanel
lists task name, executable name and status, with a divider between entries.
Back returns to Agent Communication. List, rename, stop and logs use the
same selectors and authority as the CLI. Resume creates a split terminal
and attaches there. A failed resume leaves the original terminal untouched.
