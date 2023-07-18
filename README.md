# Montage

You know montages, right? Sylvester Stalone punching meat and running up stairs? That kinda thing. Where you make progress *snap* like that. The real world doesn't work like that, which sucks, so I Frankensteined some custom software together to keep me on track.

It does some cool stuff:

- It makes me call my shots. What am I doing? Until when?
- It annoys me when I'm not doing the thing I said I was gonna do
- It lets me hook into those things to write down what I did for the day

(Some of these things are not fully implemented yet.)

To use it:

1. Be me, or at least be named Brian, or at least be OK with being called Brian by a computer.
2. Use macOS, OmniFocus for tasks, and Obsidian (or other Markdowny thing) for notes.

## How to use!

Before you start, run `montage serve` in some terminal or background job and leave it running. That's the part that manages all the state.

Then start a task! If you just wanna use the command line you can run `montage start "some thing you wanna do" --duration 25`. The number there is how long you're gonna be doing the thing.

You can also run `montage break --duration 5` (minutes again there) to take a break.

The rest is integrations.

### Vex

`montage vex` will keep track of what you're doing and start saying things every two seconds (by default) when the time you gave expires. Just keep it running in some terminal.

### Xbar

You can run `montage xbar` to get a status bar appropriate for controlling tasks in [xbar](https://xbarapp.com/) 

### OmniFocus

There's a (VERY VERY WORK IN PROGRESS) OmniFocus plugin here. Point OmniFocus at it and click buttons to start tasks and breaks.

### Watch

`montage watch` will give a debug view of whatever tasks you want. You can't use it for much more than debugging right now, and it'll probably be removed.

## License

BSD 3-Clause
