import os
import subprocess
if os.path.exists(".current_breakpoints"):
    os.remove(".current_breakpoints")
command = 'find .. -name "*.c_breakpoints*" | xargs cat > .current_breakpoints'
process = subprocess.run(command, shell=True, check=True)
