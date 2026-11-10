; Prints "Hello World", one letter per terminal flush.
;
; Each DBG is a breakpoint: run it with
;     ./target/release/vm output.bin --debug
; and the emulator pauses right after each letter appears on the screen.
; Press Space to run to the next breakpoint, S to step one instruction,
; R to restart, Q to quit.
;
; Protocol recap (terminal mapped at 0xF000):
;   buffer[0]    <- char        (STB)
;   buffer[1]    <- 3 (ETX)     (STB)
;   0xF102       <- 2 (FLUSH)   (STB) ; cursor auto-advances after each flush

MOV A, #72    ; 'H'
CALL emit

MOV A, #101   ; 'e'
CALL emit

MOV A, #108   ; 'l'
CALL emit

MOV A, #108   ; 'l'
CALL emit

MOV A, #111   ; 'o'
CALL emit

MOV A, #32    ; ' '
CALL emit

MOV A, #87    ; 'W'
CALL emit

MOV A, #111   ; 'o'
CALL emit

MOV A, #114   ; 'r'
CALL emit

MOV A, #108   ; 'l'
CALL emit

MOV A, #100   ; 'd'
CALL emit

ADD FLAGS, #1 ; halt

; prints the char held in A, then breaks
emit:
MOV B, #0x0F
MSL B, [#0 #7]
MSL B, [#0 #5]    ; B = 0xF000 (terminal buffer)
STB A, B          ; char into buffer[0]
MOV A, #3
ADD B, #1
STB A, B          ; END_OF_TEXT into buffer[1]
MOV B, #0x0F
MSL B, [#1 #4]
MSL B, [#1 #7]
MSL B, [#0 #1]    ; B = 0xF102 (command port)
MOV A, #2
STB A, B          ; FLUSH
DBG               ; <-- breakpoint: one per letter
RET
