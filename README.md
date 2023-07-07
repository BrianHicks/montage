# Montage

You know montages, right? Sylvester Stalone punching meat and running up stairs? That kinda thing. Where you make progress *snap* like that. The real world doesn't work like that, which sucks, so I Frankensteined this custom software together to keep me on track with whimsy and verve (me or the software? You'll never know!)

It does some cool stuff:

- It makes me call my shots. What am I doing? Until when?
- It annoys me when I'm not doing the thing I said I was gonna do
- It makes a killer pour-over

Some of those things are lies.

To use it:

1. Be me
2. Or don't, but use OmniFocus for tasks and Obsidian (or other Markdowny thing) for notes

## This header marks the start of the usage instructions

Before you start, run `montage vex` in some terminal or background job and leave it running. That's the bit that annoys you when there's nothing going on.

If you just wanna use the command line you can run `montage start "some thing you wanna do" --duration 25`. The number there is how long you're gonna be doing the thing. Otherwise, point OmniFocus at the script in this directory, then click the button it provides in the UI to start a thing.

You can also run `montage break --duration 5` (minutes again there) to get the noisemaker off your back for a while.

You can also run `montage xbar` to get a status bar that you can use to control the program in [xbar](https://xbarapp.com/) 

### Scripts

Montage can automatically execute some scripts in a directory. They have to be named in a specific way:

- `on-start` is run when you start doing something
- `on-break` is run when you start a break

## License

BSD 3-Clause
