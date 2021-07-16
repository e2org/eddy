lows
-----

Default view:

> files
  directories
  + new

Example fully-populated view:

> files
  directories
  ir
  rs
  e2
  vue
  javascript
  typescript
  python
  + new

Example operations:

      files : search all files · open file in editor
directories : search all directories · nav to directory
         ir : search files in ~/interreader/frontend · open file in editor
         rs : search rust projects · open src/main.rs and src/lib.rs in split editor panes
         e2 : search across files in ~/e2 · open file in editor
        vue : search .vue files · open file in editor
 javascript : search .js files · open file in editor
 typescript : search .ts files · open file in editor
     python : search .py files · open file in editor

Each flow should be selectable as soon as it's the only item matching the query -- no enter press needed.
So ideally, each flow should start with a different letter so that they may be selected in a single keypress.
The + symbol automatically selects "+ new"

F iles
D irectories
I r
R s
E 2
V ue
J avascript
T ypescript
P ython
+ new

Each flow maps to a set of standard Eddy parameters:

      files : eddy --vim --  # or: eddy --file --vim --
directories : eddy --nav --  # or: eddy --dir --nav --
         ir : eddy --sources ~/interreader/frontend --vim --
         rs : eddy --targets Cargo.toml --paths src/{main,lib}.rs --vim --
         e2 : eddy --sources ~/e2 --vim --
        vue : eddy --types vue
 javascript : eddy --types js  # or: eddy --types javascript
 typescript : eddy --types ts  # or: eddy --types typescript
     python : eddy --types py  # or: eddy --types python

      files : eddy --vim --  # or: eddy --file --vim --, eddy -fv --
directories : eddy --nav --  # or: eddy --dir --nav --, eddy -dn --
         ir : eddy ~/interreader/frontend --vim --
         rs : eddy Cargo.toml/../src/{main,lib}.rs
         e2 : eddy ~/e2 --vim --
        vue : eddy *.vue --vim --
 javascript : eddy *.js --vim --
 typescript : eddy *.ts --vim --
     python : eddy *.py --vim --

All characters typed after -- in eddy command are used as initial query.

Default action for eddy may be specified:

export EDDY_DEFAULT_ACTION="vim|emacs|vscode|open|nav|print"

These actions have corresponding shortcuts:

   --vim : -v
 --emacs : -e
--vscode : -c    # vscode is also known as simply "Code"
  --open : -o
   --nav : -n
 --print : -p
