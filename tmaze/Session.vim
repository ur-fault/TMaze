let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd D:/dev/rust/maze/tmaze
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +1 D:/dev/rust/maze/tmaze
badd +4 src/main.rs
badd +61 term://D:/dev/rust/maze//20004:C:/Windows/system32/cmd.exe
badd +36 src/game/app.rs
badd +1 src/lib.rs
badd +1 src/ui/mod.rs
badd +13 Cargo.toml
badd +18 src/settings/editable.rs
badd +221 src/settings/mod.rs
badd +82 src/ui/draw.rs
badd +174 src/ui/menu.rs
badd +81 src/ui/popup.rs
badd +45 src/ui/progressbar.rs
badd +4 d:/dev/rust/maze/cmaze/src/core/mod.rs
badd +5 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/helpers.rs
badd +15 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/layout/mod.rs
badd +76 term://D:/dev/rust/maze/tmaze//12320:C:/Windows/system32/cmd.exe
badd +14 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/frame.rs
badd +141 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/canvas.rs
badd +12 examples/menu.rs
badd +6 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/examples/popup.rs
badd +20 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/examples/rect.rs
badd +27 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/drawable/core_impl.rs
badd +121 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/layout/pos.rs
badd +87 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/layout/sized.rs
badd +1 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/layout/strings.rs
badd +17 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/cell.rs
badd +66 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/renderer.rs
badd +163 src/helpers/mod.rs
badd +8 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.0/src/lib.rs
badd +78 term://D:/dev/rust/maze/tmaze//5256:C:/Windows/system32/cmd.exe
badd +82 term://D:/dev/rust/maze/tmaze//18800:C:/Windows/system32/cmd.exe
badd +163 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.2/src/canvas.rs
badd +8 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.2/src/drawable/mod.rs
badd +15 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.2/src/frame.rs
badd +157 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.2/src/renderer.rs
badd +18 term://D:/dev/rust/maze/tmaze//14940:C:/Windows/system32/cmd.exe
badd +167 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.3/src/canvas.rs
badd +11 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.3/src/drawable/dbox.rs
badd +91 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.1.3/src/frame.rs
badd +82 term://D:/dev/rust/maze/tmaze//20420:C:/Windows/system32/cmd.exe
badd +57 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.2.1/src/layout/sized.rs
badd +61 term://D:/dev/rust/maze/tmaze//17628:C:/Windows/system32/cmd.exe
badd +49 term://D:/dev/rust/maze/tmaze//7900:C:/Windows/system32/cmd.exe
badd +70 term://D:/dev/rust/maze/tmaze//13444:C:/Windows/system32/cmd.exe
badd +70 term://D:/dev/rust/maze/tmaze//5320:C:/Windows/system32/cmd.exe
badd +2 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.3.0/src/lib.rs
badd +1 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.3.0/src/ui/mod.rs
badd +16 src/game/mod.rs
badd +1 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.4.0/src/ui/mod.rs
badd +30 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.4.0/src/ui/fullscreen_menu.rs
badd +24 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.4.0/src/ui/menu.rs
badd +1 term://D:/dev/rust/maze/tmaze//19208:C:/Windows/system32/cmd.exe
badd +845 ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/crossterm-0.27.0/src/event.rs
argglobal
%argdel
$argadd D:/dev/rust/maze/tmaze
edit src/game/app.rs
let s:save_splitbelow = &splitbelow
let s:save_splitright = &splitright
set splitbelow splitright
wincmd _ | wincmd |
vsplit
wincmd _ | wincmd |
vsplit
2wincmd h
wincmd w
wincmd w
let &splitbelow = s:save_splitbelow
let &splitright = s:save_splitright
wincmd t
let s:save_winminheight = &winminheight
let s:save_winminwidth = &winminwidth
set winminheight=0
set winheight=1
set winminwidth=0
set winwidth=1
exe 'vert 1resize ' . ((&columns * 48 + 109) / 218)
exe 'vert 2resize ' . ((&columns * 96 + 109) / 218)
exe 'vert 3resize ' . ((&columns * 72 + 109) / 218)
argglobal
if bufexists(fnamemodify("term://D:/dev/rust/maze/tmaze//19208:C:/Windows/system32/cmd.exe", ":p")) | buffer term://D:/dev/rust/maze/tmaze//19208:C:/Windows/system32/cmd.exe | else | edit term://D:/dev/rust/maze/tmaze//19208:C:/Windows/system32/cmd.exe | endif
if &buftype ==# 'terminal'
  silent file term://D:/dev/rust/maze/tmaze//19208:C:/Windows/system32/cmd.exe
endif
balt term://D:/dev/rust/maze/tmaze//5320:C:/Windows/system32/cmd.exe
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
let s:l = 73 - ((72 * winheight(0) + 38) / 76)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 73
normal! 0
wincmd w
argglobal
balt src/game/mod.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 36 - ((35 * winheight(0) + 37) / 75)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 36
normal! 0
wincmd w
argglobal
if bufexists(fnamemodify("~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.4.0/src/ui/menu.rs", ":p")) | buffer ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.4.0/src/ui/menu.rs | else | edit ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.4.0/src/ui/menu.rs | endif
if &buftype ==# 'terminal'
  silent file ~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/fyodor-0.4.0/src/ui/menu.rs
endif
balt src/helpers/mod.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let &fdl = &fdl
let s:l = 26 - ((10 * winheight(0) + 37) / 75)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 26
normal! 012|
wincmd w
2wincmd w
exe 'vert 1resize ' . ((&columns * 48 + 109) / 218)
exe 'vert 2resize ' . ((&columns * 96 + 109) / 218)
exe 'vert 3resize ' . ((&columns * 72 + 109) / 218)
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let &winminheight = s:save_winminheight
let &winminwidth = s:save_winminwidth
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
set hlsearch
nohlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
