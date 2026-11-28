# Security

Keeplight runs with your normal user rights. It connects to OpenRGB's SDK server on `127.0.0.1`, with two exceptions: when you choose **Install…** for OpenRGB or PawnIO, it downloads the tested installer over https, checks its pinned SHA-256 and runs it after Windows' admin prompt, the only time it asks for admin rights; and the window asks the release feed over https whether a newer Keeplight is out, when it opens and once a day (**Tell me when a new Keeplight is out** turns that off). The [safety model](docs/safety-model.md) lists every request it sends and every file it writes. In short:

- It never talks to lighting hardware directly; OpenRGB does.
- It never loads, saves or deletes OpenRGB profiles.
- It changes only lights you set: a device you kept, colored or stopped keeping, as the safety model lists. It writes only the files the safety model lists. New kinds of change are added to the safety model before they ship.

## Reporting a problem

Report anything that could harm a user's PC or hardware privately, through GitHub: **Security > Report a vulnerability** on this repository. Please do not open a public issue for these.

Examples worth reporting:

- the app sending OpenRGB a request the safety model does not list, especially one that changes a light;
- the app changing a light you did not set, or turning lights off;
- the app connecting anywhere other than the OpenRGB address it was given, the pinned installer links or the release feed, or writing a file the safety model does not list;
- a crafted reply from an SDK server that makes the app crash in a way that could be exploited, or run anything.

Include the app version (the bottom of the window shows it), your OpenRGB version, what you did, and what happened.

If the problem is in OpenRGB itself, for example a device that misbehaves the same way when set from OpenRGB's own window, report it to [OpenRGB](https://gitlab.com/CalcProgrammer1/OpenRGB) instead.

You should get a reply within a week. Fixes go into the next release, and the report is credited unless you would rather it was not.

## Supported versions

Only the latest release gets fixes.
