# Montage

You know montages, right? Sylvester Stalone punching meat and running up stairs? That kinda thing. Where you make progress *snap* like that. The real world doesn't work like that, which sucks, so I Frankensteined some custom software together to keep me on track.

It does some cool stuff:

- It makes me call my shots. What am I doing? Until when?
- It annoys me when I'm not doing the thing I said I was gonna do
- It lets me hook into those things to write down what I did for the day

To use it:

1. Be me, or at least be named Brian, or at least be OK with being called Brian by a computer (by default.)
2. Use macOS, OmniFocus for tasks, and Obsidian (or other Markdowny thing) for notes.

## How to use!

Before you start, run `montage serve` in some terminal or background job and leave it running. That's the part that manages all the state.

Then start a task! If you just wanna use the command line you can run `montage start "some thing you wanna do" --duration 25`. The number there is how long you're gonna be doing the thing.

You can also run `montage break --duration 5` (minutes again there) to take a break.

The rest is integrations.

### Vex

`montage vex` will keep track of what you're doing and start saying things every two seconds (by default) when the time you gave expires or you've been working too long. Just keep it running in some terminal.

`vex` can also run scripts for you if you'd like to automatically hook up things happening on the system to (e.g.) make your life more annoying or write reports somewhere automatically. See the sample scripts at `sample_scripts/` for how they'll be called.

Check out `montage vex --help` for other useful things this command can do.

### Xbar

You can run `montage xbar` to get a status bar appropriate for controlling tasks in [xbar](https://xbarapp.com/) 

### OmniFocus

There's a OmniFocus plugin here, at `Montage.omnifocusjs`. Point OmniFocus at it and click buttons to start tasks and breaks.

### Watch

`montage watch` will give a debug view of whatever tasks you want. You can't use it for much more than debugging right now, and it'll probably be removed.

### Report

`montage report` will give you a report for the day's work in Markdown, suitable for copying to a journal or log. It'll also tell you how much time was spent on tasks and short breaks (less than 15 minutes.) You can get a summary of longer breaks if you want it by passing `--include-long-breaks-in-summary`, but that doesn't tend to be super helpful information for me so I turn it off by default.

You can also call it like `montage report FIRST_DATE SECOND_DATE` to get a report for all the sessions in those two dates, inclusive.

In either case, you can pass `--no-log` or `--no-task-totals` to turn off those sections of the report.

## License

BSD 3-Clause
