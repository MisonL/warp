# Escape sequence

Escape sequence 是一组（sequence）具有特殊含义的字符，其含义通常不同于所用字符的字面含义。
在 Warp 中，我们处理 ANSI escape code，因此它们始终采用以下形式：
`<ESC> <separating char> <some combination>`

`<ESC>` 是十进制 27（0x1b）字符。
`<separating char>` 通常是 `[`，但也可以是其他字符。`<ESC><separating char>` 这一组合称为 `C1 (8-Bit) Control Characters`。
其余部分取决于实际执行的操作。它可能包含其他字符和分隔符，并按具体情况定义。一些示例操作包括 mouse tracking、在应用中移动 cursor，或处理特殊 key combination。

注意，本文档只描述**我们（代表用户）写入 pty** 的组合，不包含应用写入或从 shell 读取的其他 sequence 说明。

### 有用阅读材料

1. https://vt100.net/docs/vt100-ug/chapter3.html
2. https://www.xfree86.org/current/ctlseqs.html#PC-Style%20Function%20Keys
3. https://en.wikipedia.org/wiki/ANSI_escape_code

## C1 sequence：它们是什么，什么时候使用？

全部 C1 control character 列表见[来源](https://www.xfree86.org/current/ctlseqs.html#C1%20(8-Bit)%20Control%20Characters)：

C1 sequence | 描述
----------- | -----------
ESC D       | Index（IND 是 0x84）
ESC E       | Next Line（NEL 是 0x85）
ESC H       | Tab Set（HTS 是 0x88）
ESC M       | Reverse Index（RI 是 0x8d）
ESC N       | Single Shift Select of G2 Character Set（SS2 是 0x8e）：只影响下一个字符
ESC O       | Single Shift Select of G3 Character Set（SS3 是 0x8f）：只影响下一个字符
ESC P       | Device Control String（DCS 是 0x90）
ESC V       | Start of Guarded Area（SPA 是 0x96）
ESC W       | End of Guarded Area（EPA 是 0x97）
ESC X       | Start of String（SOS 是 0x98）
ESC Z       | Return Terminal ID（DECID 是 0x9a）。CSI c（DA）的过时形式。
ESC [       | Control Sequence Introducer（CSI 是 0x9b）
ESC \       | String Terminator（ST 是 0x9c）
ESC ]       | Operating System Command（OSC 是 0x9d）
ESC ^       | Privacy Message（PM 是 0x9e）
ESC _       | Application Program Command（APC 是 0x9f）

到目前为止，我们主要使用 2 种：CSI（ESC [）或 SS3（ESC O）。下表展示了何时使用这些 sequence 的条件：

| C1 sequence 	| terminal mode 	| modifier（shift、ctrl、alt） 	| key                                               	|
|-------------	|---------------	|-------------------------------	|---------------------------------------------------	|
| CSI         	| 任意          	| 可选                          	| 任意                                              	|
| SS3         	| APP_CURSOR    	| 不使用                        	| 方向键（上、下、右、左）<br>Home<br>End          	|

简而言之：`SS3` 只有在设置了 `TermMode::APP_CURSOR`、没有使用 modifier，并且 key 属于特定组时才能使用。否则，CSI 很可能是合适选择。

## Warp 中已覆盖的 use case

### Mouse tracking

`vim` 或 `tmux` 这类程序允许用户在应用中使用鼠标。Mouse tracking 有几种操作模式（更多见[这里](https://www.xfree86.org/current/ctlseqs.html#Mouse%20Tracking)），但 Warp 中我们关心的是 `SGR`。

基本上，之前已经实现了某种低分辨率 mouse tracking：它只能跟踪到 223 列以内的鼠标移动，这意味着在更大的 terminal window 中无法工作。2012 年起，xterm spec 引入了 `SGR`，它应该支持“更高分辨率”的 mouse tracking。每种模式都需要不同 escape sequence 来指定鼠标位置。不过，在现代环境中，可以安全假设如果 terminal emulator 支持 SGR，应用会优先使用 SGR，因此我们不关心其他 sequence。

以下是所用 sequence 的说明：

`CSI < <button> ; <column> ; <row> ; <action>`

- `<button>` 表示使用的鼠标按钮。左键是 0，右键是 2，滚轮有其他数字，拖拽或按下带 modifier 的按钮也会有其他数字。目前我们只关心左键、滚轮和鼠标拖拽。
- `<column>` 和 `<row>` 基本上是在执行 action 时鼠标指针的坐标。
- `<action>` 可以有 2 个值：`M` 表示按下和拖拽；`m` 表示释放按钮。

注意，拖拽本质上是*按下拖拽鼠标按钮*。

### Cursor movement（使用键盘）

Terminal 中的常规 cursor movement，也就是**无 modifier** 的方向键和 home/end key press action，会根据 terminal mode 表现不同。Terminal mode 基于 Warp 正在运行的程序设置；例如 `vim` 或 `emacs` 这样的长时间运行命令会设置 `APP_CURSOR` mode（它使用 CSI ? 1h sequence 设置，并使用 CSI ? 1l sequence 取消）。Warp 在 terminal_model 中跟踪该 mode（`is_term_mode_set` 方法可能有帮助）。

|                            	| Normal mode 	| APP_CURSOR mode 	|
|----------------------------	|-------------	|-----------------	|
| Previous line（上方向键）   	| CSI A       	| SS3 A           	|
| Next line（下方向键）       	| CSI B       	| SS3 B           	|
| Next char（右方向键）       	| CSI C       	| SS3 C           	|
| Previous char（左方向键）   	| CSI D       	| SS3 D           	|
| First line（home）          	| CSI H       	| SS3 H           	|
| Last line（end）            	| CSI F       	| SS3 F           	|

### 所有特殊 key 和 modifier

Function key？带 Shift 的 function key？带 Meta 或 Alt 的方向键？Shift + CMD + Key？
除非我们在 `app/src/` 代码某处显式指定了带自定义操作的 binding，否则它应由适当的 escape sequence 处理。这仍在进行中（本 README 也是如此）。每个此类 sequence 都以 `CSI` sequence 开头，后接适当组合。

如果涉及 modifier，应使用下表中的值：

| Code 	| Modifier           	|
|------	|--------------------	|
| 2    	| Shift              	|
| 3    	| Alt                	|
| 4    	| Shift + Alt        	|
| 5    	| Ctrl               	|
| 6    	| Ctrl + Shift       	|
| 7    	| Ctrl + Alt         	|
| 8    	| Ctrl + Shift + Alt 	|

例如，带 modifier 的方向键 sequence 使用以下模式：
`CSI 1 ; <modifier> <arrow code>`

（TODO：`1 ;` 具体来自哪里？）

其他 key combination 可能有不同值或完全不同的格式。最好按照上面链接的阅读材料来确定正确 sequence。
