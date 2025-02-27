> [!WARNING]
> Right now this just prints the symbols you write to gui or command line, but in the future it will probably collect that data for analys.
> This can be a security threat if logging occurs when writing passwords etc. will think of a good way of keeping stats without compromising security.
> It also require sudo privileges, because it reads directly from a device in /dev/input to listen for the keys (therefore it only works on Linux right now).

 # Purpose
The purpose of this program is for me to analyze patterns in occurence and sequence of symbols that are used when coding, and then design my symbol layer on the keyboard based on this.

# Usage
Now in the development phase you can run frekeyency by:
```bash
# Note the sudo here is required or else you wont get access to the /dev folder.
cargo build --bin frekeyency && sudo ./target/debug/frekeyency <DEVICE>
```

To use the gui (work in progress):

```bash
# Note the sudo here is required or else you wont get access to the /dev folder.
cargo build --bins && sudo ./target/debug/gui <DEVICE>
```
