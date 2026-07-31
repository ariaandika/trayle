use std::num::NonZeroU32;

/// Linux keycode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct KeyCode(NonZeroU32);

macro_rules! def {
    ($(#define $name:ident $val:literal)*) => {
        impl KeyCode {$(
            pub const $name: Self = Self(NonZeroU32::new($val).unwrap());
        )*}
    };
}

def! {
// #define RESERVED        0
#define ESC             1
#define NUM_1           2
#define NUM_2           3
#define NUM_3           4
#define NUM_4           5
#define NUM_5           6
#define NUM_6           7
#define NUM_7           8
#define NUM_8           9
#define NUM_9           10
#define NUM_0           11
#define MINUS           12
#define EQUAL           13
#define BACKSPACE       14
#define TAB             15
#define Q               16
#define W               17
#define E               18
#define R               19
#define T               20
#define Y               21
#define U               22
#define I               23
#define O               24
#define P               25
#define LEFTBRACE       26
#define RIGHTBRACE      27
#define ENTER           28
#define LEFTCTRL        29
#define A               30
#define S               31
#define D               32
#define F               33
#define G               34
#define H               35
#define J               36
#define K               37
#define L               38
#define SEMICOLON       39
#define APOSTROPHE      40
#define GRAVE           41
#define LEFTSHIFT       42
#define BACKSLASH       43
#define Z               44
#define X               45
#define C               46
#define V               47
#define B               48
#define N               49
#define M               50
#define COMMA           51
#define DOT             52
#define SLASH           53
#define RIGHTSHIFT      54
#define KPASTERISK      55
#define LEFTALT         56
#define SPACE           57
#define CAPSLOCK        58
#define F1              59
#define F2              60
#define F3              61
#define F4              62
#define F5              63
#define F6              64
#define F7              65
#define F8              66
#define F9              67
#define F10             68
#define NUMLOCK         69
#define SCROLLLOCK      70
#define KP7             71
#define KP8             72
#define KP9             73
#define KPMINUS         74
#define KP4             75
#define KP5             76
#define KP6             77
#define KPPLUS          78
#define KP1             79
#define KP2             80
#define KP3             81
#define KP0             82
#define KPDOT           83

#define ZENKAKUHANKAKU  85
#define KEY_102ND       86
#define F11             87
#define F12             88
#define RO              89
#define KATAKANA        90
#define HIRAGANA        91
#define HENKAN          92
#define KATAKANAHIRAGANA    93
#define MUHENKAN        94
#define KPJPCOMMA       95
#define KPENTER         96
#define RIGHTCTRL       97
#define KPSLASH         98
#define SYSRQ           99
#define RIGHTALT        100
#define LINEFEED        101
#define HOME            102
#define UP              103
#define PAGEUP          104
#define LEFT            105
#define RIGHT           106
#define END             107
#define DOWN            108
#define PAGEDOWN        109
#define INSERT          110
#define DELETE          111
#define MACRO           112
#define MUTE            113
#define VOLUMEDOWN      114
#define VOLUMEUP        115
#define POWER           116 /* SC System Power Down */
#define KPEQUAL         117
#define KPPLUSMINUS     118
#define PAUSE           119
#define SCALE           120 /* AL Compiz Scale (Expose) */

#define KPCOMMA         121
#define HANGEUL         122
#define HANGUEL         122
#define HANJA           123
#define YEN             124
#define LEFTMETA        125
#define RIGHTMETA       126
#define COMPOSE         127

#define STOP            128 /* AC Stop */
#define AGAIN           129
#define PROPS           130 /* AC Properties */
#define UNDO            131 /* AC Undo */
#define FRONT           132
#define COPY            133 /* AC Copy */
#define OPEN            134 /* AC Open */
#define PASTE           135 /* AC Paste */
#define FIND            136 /* AC Search */
#define CUT             137 /* AC Cut */
#define HELP            138 /* AL Integrated Help Center */
#define MENU            139 /* Menu (show menu) */
#define CALC            140 /* AL Calculator */
#define SETUP           141
#define SLEEP           142 /* SC System Sleep */
#define WAKEUP          143 /* System Wake Up */
#define FILE            144 /* AL Local Machine Browser */
#define SENDFILE        145
#define DELETEFILE      146
#define XFER            147
#define PROG1           148
#define PROG2           149
#define WWW             150 /* AL Internet Browser */
#define MSDOS           151
#define COFFEE          152 /* AL Terminal Lock/Screensaver */
#define SCREENLOCK      152
#define ROTATE_DISPLAY  153 /* Display orientation for e.g. tablets */
#define DIRECTION       153
#define CYCLEWINDOWS    154
#define MAIL            155
#define BOOKMARKS       156 /* AC Bookmarks */
#define COMPUTER        157
#define BACK            158 /* AC Back */
#define FORWARD         159 /* AC Forward */
#define CLOSECD         160
#define EJECTCD         161
#define EJECTCLOSECD    162
#define NEXTSONG        163
#define PLAYPAUSE       164
#define PREVIOUSSONG    165
#define STOPCD          166
#define RECORD          167
#define REWIND          168
#define PHONE           169 /* Media Select Telephone */
#define ISO             170
#define CONFIG          171 /* AL Consumer Control Configuration */
#define HOMEPAGE        172 /* AC Home */
#define REFRESH         173 /* AC Refresh */
#define EXIT            174 /* AC Exit */
#define MOVE            175
#define EDIT            176
#define SCROLLUP        177
#define SCROLLDOWN      178
#define KPLEFTPAREN     179
#define KPRIGHTPAREN    180
#define NEW             181 /* AC New */
#define REDO            182 /* AC Redo/Repeat */

#define F13             183
#define F14             184
#define F15             185
#define F16             186
#define F17             187
#define F18             188
#define F19             189
#define F20             190
#define F21             191
#define F22             192
#define F23             193
#define F24             194

#define PLAYCD          200
#define PAUSECD         201
#define PROG3           202
#define PROG4           203
#define ALL_APPLICATIONS    204 /* AC Desktop Show All Applications */
#define DASHBOARD       204
#define SUSPEND         205
#define CLOSE           206 /* AC Close */
#define PLAY            207
#define FASTFORWARD     208
#define BASSBOOST       209
#define PRINT           210 /* AC Print */
#define HP              211
#define CAMERA          212
#define SOUND           213
#define QUESTION        214
#define EMAIL           215
#define CHAT            216
#define SEARCH          217
#define CONNECT         218
#define FINANCE         219 /* AL Checkbook/Finance */
#define SPORT           220
#define SHOP            221
#define ALTERASE        222
#define CANCEL          223 /* AC Cancel */
#define BRIGHTNESSDOWN  224
#define BRIGHTNESSUP    225
#define MEDIA           226

#define SWITCHVIDEOMODE 227 /* Cycle between available video
                   outputs (Monitor/LCD/TV-out/etc) */
#define KBDILLUMTOGGLE  228
#define KBDILLUMDOWN    229
#define KBDILLUMUP      230

#define SEND            231 /* AC Send */
#define REPLY           232 /* AC Reply */
#define FORWARDMAIL     233 /* AC Forward Msg */
#define SAVE            234 /* AC Save */
#define DOCUMENTS       235

#define BATTERY         236

#define BLUETOOTH       237
#define WLAN            238
#define UWB             239

#define UNKNOWN         240

#define VIDEO_NEXT      241 /* drive next video source */
#define VIDEO_PREV      242 /* drive previous video source */
#define BRIGHTNESS_CYCLE    243    /* brightness up, after max is min */
#define BRIGHTNESS_AUTO 244 /* Set Auto Brightness: manual
                  brightness control is off,
                  rely on ambient */
#define BRIGHTNESS_ZERO 244
#define DISPLAY_OFF     245 /* display device to off state */

#define WWAN            246 /* Wireless WAN (LTE, UMTS, GSM, etc.) */
#define WIMAX           246
#define RFKILL          247 /* Key that controls all radios */

#define MICMUTE         248 /* Mute / unmute the microphone */
}
