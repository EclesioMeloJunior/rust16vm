; ==========================================================================
; donut.s - a spinning, shaded, perspective-projected 3D ASCII torus
;
;   cargo build --release                  (release is REQUIRED: the math
;                                           relies on wrapping u16 arithmetic)
;   ./target/release/asm output.bin ./testdata/donut.s
;   ./target/release/vm output.bin         (Space = run, Q = quit)
;
; The picture is the classic donut.c torus (R=2, r=1, camera at distance 5,
; light from above/behind the viewer, 12 luminance chars  .,-~:;=!*#$@ ),
; computed with 16-bit integers only.
;
; HOW IT WORKS
;   The animation is a loop of 48 frames: the tumble angle A advances pi/48
;   per frame and the in-plane angle B advances 2pi/48 per frame, so after 48
;   frames the torus is back where it started (a torus maps onto itself after
;   half a turn of A).
;
;   The emulator executes only ~2000 instructions per UI tick, and a frame
;   costs ~120k instructions to render, so every frame is rendered ONCE into
;   its own 1056-byte slot (48 slots = 50KB of the 64KB memory). The first
;   lap is therefore slow (about a frame per second); from the second lap on
;   the frames are just copied from memory to the terminal and the donut
;   spins smoothly.
;
;   Rendering a frame, with c=sin(phi) l=cos(phi) d=cos(theta) f=sin(theta)
;   e=sinA g=cosA n=sinB m=cosB and h=2+d (all values fixed point):
;       x = h*P + f*e*n      P = l*m - c*g*n        (per phi)
;       y = h*Q - f*e*m      Q = l*n + c*g*m
;       z = h*S + f*g + 5    S = c*e
;       L = d*V + f*U        V = -(c*(g*m+e) + l*n) U = e*m - g
;   so the inner loop (over theta) needs one multiply per coordinate: the
;   theta-only terms come from a 32-record table rebuilt once per frame.
;   1/z comes from a lookup table, then col = 22 + 27*x/z, row = 11 + 13.5*y/z.
;   Hidden surfaces are removed with a z-buffer; back faces are skipped early.
;   Samples are 96 (phi) x 32 (theta) and each one paints 2 cells (3 when
;   very close), then a small z-aware pass closes the remaining pin holes.
;
; ISA NOTES this file is careful about
;   * immediates truncate silently: MOV 8 bits, ADD/SUB/MUL/DIV 6 bits,
;     compares 5 bits, LDR/STR offsets 5 bits, MSL/MSR 5-bit value/3-bit shift
;   * shifts and compares are unsigned: signed values are biased first
;   * immediate jump targets are 11 bits: the program must stay below 0x0800
;   * CALL keeps its return address in M (no nesting, M is scratch otherwise)
;
; REGISTERS
;   SP = 0x1500 for the whole program: base pointer of the global variables
;   BP = second pointer (0x1540 trig block / record pointer / end pointer)
;   A, B, C, M = scratch
;
; MEMORY MAP
;   0x0800 SIN    120 words: 128*sin(2*pi*t/96), t = 0..119 (cos = t+24)
;   0x0900 CHARS  512 bytes: luminance>>7 -> character
;   0x0B00 OOZ    4 bytes per depth step: word 27*16/z, word 22528-256*(27*16/z)
;   0x1300 REC    32 theta records of 14 bytes: h,Fz,Fx,Fy,d,LF,Cz
;   0x1500 globals (SP):  0 S   2 T   4 P   6 Q   8 OOZ base  10 ZBUF base
;                        12 V  14 CHARS base  16 D = slot-zbuf+1
;                        18 END (record end for this phi)  20 I  22 K (frame)
;                        24 SLOT  26 BAKED  28 end of record group A
;                        30 end of all records
;   0x1540 trig block (BP): 0 e  2 g  4 n  6 m  8 gn  10 gm  12 en  14 em
;                        16 U  18 gm+e  20 0x8040  22 256  24 0x8000
;                        26 0x4000  28,30 temporaries
;   0x1600 ZBUF   1056 bytes (+ padding), one byte per cell: 16*27/z, 0 = empty
;   0x1B00 SLOTS  48 frames x 22 rows x 48 bytes (47 chars + newline)
;   0xF000 terminal: buffer 0xF000, cursor word 0xF100, command 0xF102
;          (rows 0..21 = the donut, row 23 = caption during the first lap)
; ==========================================================================

start:
    MOV SP, #84
    MSL SP, [#0 #6]         ; SP = 0x1500 (globals), never changes again
    MOV A, #44
    MSL A, [#0 #6]
    STR A, [SP #8]          ; OOZ base  = 0x0B00
    MOV A, #88
    MSL A, [#0 #6]
    STR A, [SP #10]         ; ZBUF base = 0x1600
    MOV A, #36
    MSL A, [#0 #6]
    STR A, [SP #14]         ; CHARS base = 0x0900
    MOV A, #80
    MSL A, [#0 #6]
    ADD A, #38
    STR A, [SP #28]         ; 0x1426 = REC + 21*14 (end of record group A)
    MOV A, #83
    MSL A, [#0 #6]
    STR A, [SP #30]         ; 0x14C0 = REC + 32*14

; ---- SIN table: first quarter as literals --------------------------------
    MOV B, #32
    MSL B, [#0 #6]          ; B = 0x0800 (entry 0 is already 0)
    MOV A, #8
    STR A, [B #2]
    MOV A, #17
    STR A, [B #4]
    MOV A, #25
    STR A, [B #6]
    MOV A, #33
    STR A, [B #8]
    MOV A, #41
    STR A, [B #10]
    MOV A, #49
    STR A, [B #12]
    MOV A, #57
    STR A, [B #14]
    MOV A, #64
    STR A, [B #16]
    MOV A, #71
    STR A, [B #18]
    MOV A, #78
    STR A, [B #20]
    MOV A, #84
    STR A, [B #22]
    MOV A, #91
    STR A, [B #24]
    MOV A, #96
    STR A, [B #26]
    MOV A, #102
    STR A, [B #28]
    MOV A, #106
    STR A, [B #30]
    ADD B, #32              ; B = &sin[16]
    MOV A, #111
    STR A, [B #0]
    MOV A, #115
    STR A, [B #2]
    MOV A, #118
    STR A, [B #4]
    MOV A, #121
    STR A, [B #6]
    MOV A, #124
    STR A, [B #8]
    MOV A, #126
    STR A, [B #10]
    MOV A, #127
    STR A, [B #12]
    MOV A, #128
    STR A, [B #14]
    STR A, [B #16]          ; sin[24] = 128
    ; second quarter: sin[t] = sin[48-t], t = 25..47
    MOV A, B
    ADD A, #14              ; A = &sin[23]
    ADD B, #18              ; B = &sin[25]
    MOV M, #134
    MSL M, [#0 #4]          ; M = 0x0860 = &sin[48]
sin_mirror:
    LDR C, [A #0]
    STR C, [B #0]
    SUB A, #2
    ADD B, #2
    LT B, M
    CJP sin_mirror
    ; rest: sin[t] = -sin[t-48], t = 48..119
    MOV A, #32
    MSL A, [#0 #6]          ; A = &sin[0]
    MOV M, #143
    MSL M, [#0 #4]          ; M = 0x08F0 = &sin[120]
sin_neg:
    LDR C, [A #0]
    MOV BP, #0
    SUB BP, C               ; two's complement negation
    STR BP, [B #0]
    ADD A, #2
    ADD B, #2
    LT B, M
    CJP sin_neg

; ---- constants used by the fixed point helpers ---------------------------
    MOV BP, #85
    MSL BP, [#0 #6]         ; BP = 0x1540 (trig block)
    MOV A, #128
    MSL A, [#1 #2]
    MSL A, [#0 #6]
    STR A, [BP #20]         ; 0x8040 (sign bias + rounding for >>7)
    MOV A, #128
    MSL A, [#0 #1]
    STR A, [BP #22]         ; 256
    MSL A, [#0 #6]
    STR A, [BP #26]         ; 0x4000
    MSL A, [#0 #1]
    STR A, [BP #24]         ; 0x8000

; ---- CHARS: 512 bytes, index = (luminance + bias) >> 7 --------------------
    MOV B, #36
    MSL B, [#0 #6]          ; B = 0x0900
    MOV A, #46              ; '.' for everything that faces away from the light
    MOV C, #255
    ADD C, #17              ; 272 entries
    CALL fill
    MOV A, #44              ; ','
    MOV C, #16
    CALL fill
    MOV A, #45              ; '-'
    MOV C, #16
    CALL fill
    MOV A, #126             ; '~'
    MOV C, #16
    CALL fill
    MOV A, #58              ; ':'
    MOV C, #16
    CALL fill
    MOV A, #59              ; ';' (written as a number: ';' starts a comment)
    MOV C, #16
    CALL fill
    MOV A, #61              ; '='
    MOV C, #16
    CALL fill
    MOV A, #33              ; '!'
    MOV C, #16
    CALL fill
    MOV A, #42              ; '*'
    MOV C, #16
    CALL fill
    MOV A, #35              ; '#'
    MOV C, #16
    CALL fill
    MOV A, #36              ; '$'
    MOV C, #16
    CALL fill
    MOV A, #64              ; '@'
    MOV C, #80
    CALL fill

; ---- OOZ table: for depth step zi (z = zi/64 + 1), zi = 40..472 ----------
;      word 0 = ooz = round(27*1024 / (zi+64))   (= 16 * 27/z)
;      word 2 = 22528 - 256*ooz   (screen centre minus the +4.0 bias of x, y)
    MOV B, #46
    MSL B, [#0 #6]
    ADD B, #32              ; B = 0x0BA0 = OOZ + 4*40
    MOV C, #104             ; C = zi + 64
ooz_loop:
    MOV A, #108
    MSL A, [#0 #7]
    MSL A, [#0 #1]          ; A = 27648
    MOV M, C
    MSR M, [#0 #1]
    ADD A, M
    DIV A, C                ; rounded division
    STR A, [B #0]
    MSL A, [#0 #7]
    MSL A, [#0 #1]          ; 256 * ooz
    MOV M, #88
    MSL M, [#0 #7]
    MSL M, [#0 #1]          ; 22528 = 21.5*1024 + 512
    SUB M, A
    STR M, [B #2]
    ADD B, #4
    ADD C, #1
    MOV A, #134
    MSL A, [#1 #2]          ; 537
    LT C, A
    CJP ooz_loop

; ==========================================================================
; main loop: one iteration per displayed frame, K = 0..47
; ==========================================================================
frame_loop:
    LDR A, [SP #22]         ; K
    MUL A, #33
    MSL A, [#0 #5]          ; K * 1056
    MOV B, #108
    MSL B, [#0 #6]          ; 0x1B00
    ADD A, B
    STR A, [SP #24]         ; SLOT = address of this frame's picture
    LDR B, [SP #26]
    MOV C, #48
    GTE B, C                ; all 48 frames rendered already?
    CJP replay

; ---- tell the user what is going on: "rendering frame NN/48 ..." ----------
    CALL caption_at
    MOV A, #52          ; '4'
    STB A, [B #19]
    MOV A, #56          ; '8'
    STB A, [B #20]
    MOV A, #114         ; 'r'
    STB A, [B #0]
    STB A, [B #5]
    STB A, [B #11]
    STB A, [B #26]
    MOV A, #101         ; 'e'
    STB A, [B #1]
    STB A, [B #4]
    STB A, [B #14]
    MOV A, #110         ; 'n'
    STB A, [B #2]
    STB A, [B #7]
    MOV A, #100         ; 'd'
    STB A, [B #3]
    MOV A, #105         ; 'i'
    STB A, [B #6]
    STB A, [B #25]
    MOV A, #103         ; 'g'
    STB A, [B #8]
    MOV A, #32          ; ' '
    STB A, [B #9]
    STB A, [B #15]
    STB A, [B #21]
    STB A, [B #23]
    STB A, [B #29]
    MOV A, #102         ; 'f'
    STB A, [B #10]
    STB A, [B #24]
    MOV A, #97          ; 'a'
    STB A, [B #12]
    STB A, [B #31]
    MOV A, #109         ; 'm'
    STB A, [B #13]
    MOV A, #47          ; '/'
    STB A, [B #18]
    MOV A, #45          ; '-'
    STB A, [B #22]
    MOV A, #115         ; 's'
    STB A, [B #27]
    MOV A, #116         ; 't'
    STB A, [B #28]
    MOV A, #108         ; 'l'
    STB A, [B #30]
    LDR A, [SP #22]
    ADD A, #1               ; frame number 1..48
    MOV C, A
    DIV C, #10              ; tens
    MOV M, C
    MUL M, #10
    SUB A, M                ; ones
    ADD C, #48
    STB C, [B #16]
    ADD A, #48
    STB A, [B #17]
    ADD B, #32
    MOV A, #112         ; 'p'
    STB A, [B #0]
    MOV A, #32          ; ' '
    STB A, [B #1]
    STB A, [B #4]
    MOV A, #105         ; 'i'
    STB A, [B #2]
    MOV A, #115         ; 's'
    STB A, [B #3]
    STB A, [B #5]
    MOV A, #108         ; 'l'
    STB A, [B #6]
    MOV A, #111         ; 'o'
    STB A, [B #7]
    MOV A, #119         ; 'w'
    STB A, [B #8]
    ADD B, #9
    CALL term_flush

; ---- render frame K into its slot ----------------------------------------
    LDR A, [SP #24]
    LDR B, [SP #10]         ; B = ZBUF
    MOV C, A
    SUB C, B
    ADD C, #1
    STR C, [SP #16]         ; D: picture address = z-buffer address + D
    ; clear the slot (spaces) and the z-buffer (0 = infinitely far)
    MOV BP, #33
    MSL BP, [#0 #5]
    ADD BP, A               ; BP = end of slot
    MOV C, #32
    MSL C, [#0 #7]
    MSL C, [#0 #1]
    ADD C, #32              ; C = 0x2020 (two spaces)
    MOV M, #0
clear_loop:
    STR C, [A #0]
    STR M, [B #0]
    STR C, [A #2]
    STR M, [B #2]
    STR C, [A #4]
    STR M, [B #4]
    STR C, [A #6]
    STR M, [B #6]
    STR C, [A #8]
    STR M, [B #8]
    STR C, [A #10]
    STR M, [B #10]
    STR C, [A #12]
    STR M, [B #12]
    STR C, [A #14]
    STR M, [B #14]
    ADD A, #16
    ADD B, #16
    LT A, BP
    CJP clear_loop
    ; every row ends with a newline (column 47)
    LDR A, [SP #24]
    ADD A, #47
    MOV C, #10
newline_loop:
    STB C, [A #0]
    ADD A, #48
    LT A, BP
    CJP newline_loop

    ; ---- trig constants of this frame: A index = K, B index = 2K ---------
    MOV BP, #85
    MSL BP, [#0 #6]         ; BP = trig block
    LDR A, [SP #22]
    ADD A, A
    MOV B, #32
    MSL B, [#0 #6]          ; B = SIN
    ADD A, B                ; A = &sin[K]
    LDR C, [A #0]
    STR C, [BP #0]          ; e = sin A
    ADD A, #48
    LDR C, [A #0]
    STR C, [BP #2]          ; g = cos A
    LDR A, [SP #22]
    MSL A, [#0 #2]
    ADD A, B                ; A = &sin[2K]
    LDR C, [A #0]
    STR C, [BP #4]          ; n = sin B
    ADD A, #48
    LDR C, [A #0]
    STR C, [BP #6]          ; m = cos B
    LDR A, [BP #2]
    LDR C, [BP #4]
    MUL A, C
    CALL sar7
    STR A, [BP #8]          ; gn
    LDR A, [BP #2]
    LDR C, [BP #6]
    MUL A, C
    CALL sar7
    STR A, [BP #10]         ; gm
    LDR C, [BP #0]
    ADD A, C
    STR A, [BP #18]         ; gm + e
    LDR A, [BP #0]
    LDR C, [BP #4]
    MUL A, C
    CALL sar7
    STR A, [BP #12]         ; en
    LDR A, [BP #0]
    LDR C, [BP #6]
    MUL A, C
    CALL sar7
    STR A, [BP #14]         ; em
    LDR C, [BP #2]
    SUB A, C
    STR A, [BP #16]         ; U = em - g

    ; ---- the 32 theta records; record r is theta = (r - 10) * 2pi/32 -----
    ;      +0 h = (256+d)/2          (2 + cos, Q6)
    ;      +2 Fz = f*g/2  + 4.0      (Q13, biased)
    ;      +4 Fx = f*en/2 + 4.0
    ;      +6 Fy = -f*em/2 + 4.0
    ;      +8 d                      (cos, Q7)
    ;      +10 LF = f*U + 0x8000     (light, biased)
    ;      +12 Cz = Fz + (2d+1)/5    (back-face test term)
    MOV A, #34
    MSL A, [#4 #6]          ; 0x0884 = &sin[66]  (theta index -30 mod 96)
    STR A, [BP #28]
    MOV B, #76
    MSL B, [#0 #6]          ; B = REC = 0x1300
rec_loop:
    LDR A, [BP #28]
    ADD A, #48
    LDR C, [A #0]           ; C = d
    STR C, [B #8]
    LDR A, [BP #22]
    ADD A, C
    MSR A, [#0 #1]
    STR A, [B #0]           ; h
    MUL C, #26
    MOV A, #102
    MSL A, [#6 #4]          ; 1638 = 8192/5
    ADD C, A
    STR C, [B #12]          ; (2d+1)/5 in Q13, Fz is added below
    LDR A, [BP #28]
    LDR A, [A #0]           ; A = f
    LDR C, [BP #2]
    MUL A, C
    CALL half
    STR A, [B #2]           ; Fz
    LDR C, [B #12]
    ADD A, C
    STR A, [B #12]          ; Cz
    LDR A, [BP #28]
    LDR A, [A #0]
    LDR C, [BP #12]
    MUL A, C
    CALL half
    STR A, [B #4]           ; Fx
    LDR A, [BP #28]
    LDR A, [A #0]
    LDR C, [BP #14]
    MUL A, C
    MOV C, #0
    SUB C, A
    MOV A, C
    CALL half
    STR A, [B #6]           ; Fy
    LDR A, [BP #28]
    LDR A, [A #0]
    LDR C, [BP #16]
    MUL A, C
    LDR C, [BP #24]
    ADD A, C
    STR A, [B #10]          ; LF
    LDR A, [BP #28]
    ADD A, #6               ; next theta (3 table steps)
    MOV C, #35
    MSL C, [#0 #6]          ; 0x08C0 = &sin[96]
    LT A, C
    CJP rec_nowrap
    MOV C, #192
    SUB A, C
rec_nowrap:
    STR A, [BP #28]
    ADD B, #14
    LDR A, [SP #30]
    LT B, A
    CJP rec_loop

    ; ---- for every phi: I = 0..95 ----------------------------------------
    MOV A, #0
    STR A, [SP #20]
phi_loop:
    MOV BP, #85
    MSL BP, [#0 #6]         ; BP = trig block
    LDR A, [SP #20]
    ADD A, A
    MOV B, #32
    MSL B, [#0 #6]
    ADD B, A                ; B = &sin[I]
    LDR A, [B #0]
    STR A, [BP #28]         ; c
    ADD B, #48
    LDR A, [B #0]
    STR A, [BP #30]         ; l
    LDR C, [BP #6]
    MUL A, C                ; l*m
    LDR B, [BP #28]
    LDR C, [BP #8]
    MUL B, C                ; c*gn
    SUB A, B
    CALL sar7
    STR A, [SP #4]          ; P
    LDR A, [BP #30]
    LDR C, [BP #4]
    MUL A, C                ; l*n
    LDR B, [BP #28]
    LDR C, [BP #10]
    MUL B, C                ; c*gm
    ADD A, B
    CALL sar7
    STR A, [SP #6]          ; Q
    LDR A, [BP #30]
    LDR C, [BP #4]
    MUL A, C                ; l*n
    LDR B, [BP #28]
    LDR C, [BP #18]
    MUL B, C                ; c*(gm+e)
    ADD A, B
    MOV B, #0
    SUB B, A
    MOV A, B
    CALL sar7
    STR A, [SP #12]         ; V
    LDR A, [BP #28]
    LDR C, [BP #0]
    MUL A, C                ; c*e
    CALL sar7
    STR A, [SP #0]          ; S
    MSL A, [#0 #7]
    MOV C, #131
    MSL C, [#3 #4]
    MSL C, [#3 #4]          ; 0x8333 = 4.0 + 0.1 margin (Q13)
    ADD A, C
    STR A, [SP #2]          ; T = 4.1 + 2*S : back-face threshold
    ; even phi: all 32 records; odd phi: only the 21 outer ones (the inner
    ; part of the tube has a smaller radius and needs fewer samples)
    LDR A, [SP #20]
    MOV B, A
    MSR B, [#0 #1]
    MSL B, [#0 #1]
    LDR C, [SP #30]
    EQ A, B
    CJP phi_even
    LDR C, [SP #28]
phi_even:
    STR C, [SP #18]         ; END
    MOV BP, #76
    MSL BP, [#0 #6]         ; BP = first record

; ---- inner loop: one sample of the surface -------------------------------
theta_loop:
    LDB A, [BP #0]          ; A = h
    LDR C, [SP #0]
    MUL C, A                ; C = h*S
    LDR M, [BP #12]
    ADD M, C
    LDR B, [SP #2]
    GTE M, B                ; faces away from the camera -> skip
    CJP theta_next
    LDR M, [BP #2]
    ADD C, M                ; C = z - 1          (Q13)
    LDR M, [SP #4]
    MUL M, A
    LDR B, [BP #4]
    ADD M, B
    MSR M, [#0 #7]          ; M = x + 4          (Q6)
    LDR B, [SP #6]
    MUL B, A
    LDR A, [BP #6]
    ADD B, A
    MSR B, [#0 #7]          ; B = y + 4          (Q6)
    MSR C, [#0 #7]
    MSL C, [#0 #2]
    LDR A, [SP #8]
    ADD C, A                ; C = OOZ record of this depth
    LDR A, [C #0]           ; A = 16*27/z  (also the z-buffer value)
    LDR C, [C #2]
    MUL M, A
    ADD M, C
    MSR M, [#0 #7]
    MSR M, [#0 #3]          ; M = column
    MUL B, A
    ADD B, C
    MSR B, [#0 #7]
    MSR B, [#0 #4]          ; B = row
    MUL B, #48
    ADD B, M
    LDR C, [SP #10]
    ADD C, B                ; C = z-buffer cell
    LDR M, [BP #8]
    LDR B, [SP #12]
    MUL M, B                ; d*V
    LDR B, [BP #10]
    ADD M, B                ; + f*U + bias
    MSR M, [#0 #7]
    LDR B, [SP #14]
    ADD M, B                ; M = address of the luminance character
    LDB B, [C #0]
    GT B, A                 ; something closer already there?
    CJP cell_1
    STB A, [C #0]
    LDR B, [SP #16]
    ADD B, C
    CPY M, B
cell_1:
    LDB B, [C #1]
    GT B, A
    CJP cell_2
    STB A, [C #1]
    LDR B, [SP #16]
    ADD B, C
    ADD B, #1
    CPY M, B
cell_2:
    MOV B, #125
    LT A, B                 ; third cell only for very close samples
    CJP theta_next
    LDB B, [C #2]
    GT B, A
    CJP theta_next
    STB A, [C #2]
    LDR B, [SP #16]
    ADD B, C
    ADD B, #2
    CPY M, B
theta_next:
    ADD BP, #14
    LDR A, [SP #18]
    LT BP, A
    CJP theta_loop

    LDR A, [SP #20]
    ADD A, #1
    STR A, [SP #20]
    MOV B, #96
    LT A, B
    CJP phi_loop

; ---- close pin holes: a cell clearly farther than both its left and right
;      neighbours (or a 2-cell gap) shows the neighbour's character ----------
    LDR B, [SP #10]         ; B = &z[q-1]
    MOV BP, #131
    MSL BP, [#5 #3]         ; 1053
    ADD BP, B
hfix_loop:
    LDB A, [B #1]
    ADD A, #8               ; A = z[q] + tolerance
    LDB C, [B #0]
    LTE C, A
    CJP hfix_next
    LDB C, [B #2]
    GT C, A
    CJP hfix_one
    ADD C, #8
    LDB M, [B #3]
    LTE M, A
    CJP hfix_next
    LTE M, C
    CJP hfix_next
    LDR A, [SP #16]
    ADD A, B                ; A = picture cell q-1
    MOV C, A
    ADD C, #1
    CPY A, C                ; q   <- q-1
    ADD A, #3
    ADD C, #1
    CPY A, C                ; q+1 <- q+2
    JMP hfix_next
hfix_one:
    LDR A, [SP #16]
    ADD A, B
    MOV C, A
    ADD C, #1
    CPY A, C
hfix_next:
    ADD B, #1
    LT B, BP
    CJP hfix_loop
    ; same idea vertically
    LDR B, [SP #10]         ; B = &z[q-48]
    MOV BP, #15
    MSL BP, [#0 #6]         ; 960
    ADD BP, B
vfix_loop:
    MOV A, B
    ADD A, #48
    LDB M, [A #0]
    ADD M, #8               ; M = z[q] + tolerance
    LDB C, [B #0]
    LTE C, M
    CJP vfix_next
    ADD A, #48
    LDB C, [A #0]
    LTE C, M
    CJP vfix_next
    LDR A, [SP #16]
    ADD A, B
    MOV C, A
    ADD C, #48
    CPY A, C                ; q <- q-48
vfix_next:
    ADD B, #1
    LT B, BP
    CJP vfix_loop

    LDR A, [SP #26]
    ADD A, #1
    STR A, [SP #26]         ; BAKED++
    MOV B, #48
    LT A, B
    CJP show
    CALL caption_at         ; last frame rendered: wipe the caption
    MOV A, #32
    MOV C, #41
    CALL fill
    CALL term_flush
    JMP show

; ---- replay: the frame is already in memory, just keep the pace ----------
replay:
    MOV A, #180
    MSL A, [#0 #4]          ; 2880 rounds
delay_loop:
    SUB A, #1
    NEQ A, #0
    CJP delay_loop

; ---- copy the slot to the terminal: 4 rows (192 bytes) per flush ---------
show:
    MOV B, #241
    MSL B, [#0 #7]
    MSL B, [#0 #1]          ; 0xF100 cursor
    MOV A, #0
    STR A, [B #0]           ; X = 0, Y = 0
    LDR A, [SP #24]         ; A = source
    MOV BP, #33
    MSL BP, [#0 #5]
    ADD BP, A               ; BP = end of slot
chunk_loop:
    MOV B, #240
    MSL B, [#0 #7]
    MSL B, [#0 #1]          ; 0xF000 terminal buffer
    MOV M, #12              ; 12 * 16 bytes
block_loop:
    LDR C, [A #0]
    STR C, [B #0]
    LDR C, [A #2]
    STR C, [B #2]
    LDR C, [A #4]
    STR C, [B #4]
    LDR C, [A #6]
    STR C, [B #6]
    LDR C, [A #8]
    STR C, [B #8]
    LDR C, [A #10]
    STR C, [B #10]
    LDR C, [A #12]
    STR C, [B #12]
    LDR C, [A #14]
    STR C, [B #14]
    ADD A, #16
    ADD B, #16
    EQ A, BP
    CJP chunk_flush
    SUB M, #1
    NEQ M, #0
    CJP block_loop
chunk_flush:
    CALL term_flush         ; the cursor continues on the next row
    LT A, BP
    CJP chunk_loop

    ; next frame
    LDR A, [SP #22]
    ADD A, #1
    MOV B, #48
    LT A, B
    CJP frame_keep
    MOV A, #0
frame_keep:
    STR A, [SP #22]
    JMP frame_loop

; ==========================================================================
; helpers (CALL / RET, return address in M; they need BP = trig block)
; ==========================================================================

; A = round(A / 128) for a signed A; clobbers C
sar7:
    LDR C, [BP #20]
    ADD A, C                ; + 0x8000 (make it positive) + 64 (rounding)
    MSR A, [#0 #7]
    LDR C, [BP #22]
    SUB A, C                ; remove the shifted bias
    RET

; A = A/2 + 4.0 (Q13) for a signed A; clobbers C
half:
    LDR C, [BP #24]
    ADD A, C
    MSR A, [#0 #1]
    LDR C, [BP #26]
    ADD A, C
    RET

; end the text at B and flush the terminal buffer; clobbers B, C
term_flush:
    MOV C, #3
    STB C, [B #0]           ; end of text
    MOV B, #241
    MSL B, [#0 #7]
    MSL B, [#2 #1]          ; 0xF102 command port
    MOV C, #2
    STB C, [B #0]           ; flush
    RET

; cursor to column 3 of row 23, B = terminal buffer; clobbers A
caption_at:
    MOV B, #241
    MSL B, [#0 #7]
    MSL B, [#0 #1]          ; 0xF100 cursor (low byte X, high byte Y)
    MOV A, #23
    MSL A, [#0 #7]
    MSL A, [#3 #1]
    STR A, [B #0]
    MOV B, #240
    MSL B, [#0 #7]
    MSL B, [#0 #1]          ; 0xF000
    RET

; store C copies of byte A at B (B advances); clobbers C
fill:
    STB A, [B #0]
    ADD B, #1
    SUB C, #1
    NEQ C, #0
    CJP fill
    RET
