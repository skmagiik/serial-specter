# Serial-Specter

<img src="serial-specter-logo.png" alt="Serial Specter logo, a cute spy ghost" width="200"/>

## Overview

🚨 This project is a work in progress. There may be bugs or missing/incomplete features. 🚨

Serial-Specter 👻 is a command-line tool written in rust that allows you to snoop and listen in between two serial communication devices. It is useful both for offensive and defensive security testing.
Currently, this is a passive listening tool that is well suited for data capture and analysis of raw serial communications.

#### Key Features:
- RS232, TTL support. Other serial com ports may work, give them a try!
- Color-coded output to easily identify the data's source device
- ASCII, hexdump, or xxd output formats


#### Roadmap / Desired Features:
- [ ] Live start/stop data frame system
- [ ] Output printing start/stop trigger on data bytes or data frames
- [ ] In-transit Regex or script-based data replacement
  - triggered on data bytes or data frames
- [ ] Serial data recording and playback
  - parameter replacement