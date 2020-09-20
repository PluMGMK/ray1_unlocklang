ray1_unlocklang
===============

Yay, the first use-case for my [pmw1-rs crate](https://github.com/PluMGMK/pmw1-rs)! It could probably use a better name though...

Anyway, this is a very simple patcher for the RAYMAN.EXE file that ships with [GOG's version of "Rayman Forever"](https://www.gog.com/game/rayman_forever), to make it honour language settings instead of defaulting to English.

# The problem

Let's say you have the GOG version of Rayman 1, and you use the [Rayman Control Panel](https://github.com/RayCarrot/RayCarrot.RCP.Metro) (or similar) to change its language setting from the default "English" to "French" (or "German"). You proceed to start up the game, and are greeted (after the opening sequence) by the game's main menu… still in English! What's more, you go back to the RCP, and find that the setting has magically reset itself to "English"!

# The solution

Let's suppose you have no access to any version other than the GOG one, but you really want to play it in French (or German). That's where this tool comes in!

If you're comfortable with the command line:
```
$ git clone https://github.com/PluMGMK/ray1_unlocklang.git
$ cd ray1_unlocklang
$ cargo run --release -- /PATH/TO/RAYMAN1.EXE
```

Not your cup of tea? Or maybe you just don't have a Rust nightly toolchain installed? No problem, you can also grab the pre-compiled EXE from the Releases page. On Windows, you can just drag your `RAYMAN1.EXE` file onto `ray1_unlocklang.exe` in the File Explorer and it'll automatically get patched.

Either way, a `RAYMAN1.EXE.BAK` file will be created just in case.

Now, if you go to the Rayman Control Panel, set the language setting to something other than "English", and then run the game, you'll find that it actually runs in the language that you have selected! It'll still say "`RAYMAN (US) 1.21`" in the version string when it's starting up, but that doesn't matter. The actual game will be in your chosen language.

# How it works

As pointed out [here](https://raymanpc.com/forum/viewtopic.php?f=89&t=25761&p=1418421&hilit=hard+coded#p1418421), the GOG version actually includes data and code for all three languages, but there is code in there to force it to use English. This patcher changes ten bytes of machine code in the `LOAD_CONFIG` function to replace the instructions forcing English with an instruction honouring the setting, like in other versions of Rayman 1. You can see the instructions in the comments of the source code for this patcher.

To help understand it, here are screenshots of the relevant portion of `LOAD_CONFIG` from IDA Version 7.0.191002 (Freeware version), before and after patching:
![Before](https://github.com/PluMGMK/ray1_unlocklang/blob/master/before.png?raw=true)
![After](https://github.com/PluMGMK/ray1_unlocklang/blob/master/after.png?raw=true)

Before patching, the code creates zero values (corresponding to the "English" language setting) in `bx` and `edx`, then uses them to set the global language variable to zero, and call `LoadLanguageTxt` with zero as its argument. The patch causes it to instead call `LoadLanguageTxt` with the value loaded *from* the global language variable, like other versions of the game. The four `nop`s are there to pad it out to the full ten bytes of code.

A complication to this exercise is that Rayman 1 is in fact a compressed executable in the PMW1 format. Therefore, this program uses my [pmw1-rs crate](https://github.com/PluMGMK/pmw1-rs) to decompress the relevant piece of the EXE, patch it, and then recompress it. If that sounds interesting, you can check out that crate's source code and documentation too.
