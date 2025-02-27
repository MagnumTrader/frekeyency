> [!WARNING]
> Right now this just prints the symbols you write to gui or command line, but in the future it will probably collect that data for analys.
> This can be a security threat if logging occurs when writing passwords etc. 
> I will think of a good solution of keeping stats without compromising security.
> It also require sudo privileges, because it reads directly from a device in /dev/input to listen for the keys (therefore it only works on Linux right now).

 # Purpose
The purpose of this program is for me to analyze patterns in occurence and sequence of symbols that are used when coding, and then design my symbol layer on the keyboard based on this.

# Usage
You can run frekeyency by:
```bash
# Note the sudo here is required or else you wont get access to the /dev folder.
cargo build --bin frekeyency && sudo ./target/debug/frekeyency <DEVICE>
```

To use the gui (work in progress):
This starts frekeyency as a child process and you interact with that process through its stdin/out
```bash
# sudo also required here because of /dev/input access
cargo build --bins && sudo ./target/debug/gui <DEVICE>
```
