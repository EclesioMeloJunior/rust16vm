MOV B, #8
MOV A, #1

loop: 
LTE B, #1
CJP END
MUL A, B
SUB B, #1
JMP loop

END:
JMP int_to_str

return_to_loop:
MOV A, #2
MOV B, #482
MSL B, [#2 #7]
STR A, B

ADD FLAGS, #1

; get the algarism, sum to 48
; to get the correct ascii repr
; and place in the buffer
int_to_str:
GT A, #9
CJP int_to_str_bef_loop

MOV B, #480
MSL B, [#0 #7]

ADD A, #48
STB A, B
ADD B, #1

; indicates end_of_text
MOV A, #3
STB A, B
JMP return_to_loop

int_to_str_bef_loop:
MOV C, #0

int_to_str_loop:
LTE A, #0 
CJP reverse_buffer

MOV B, #0x0F
MSL B, [#0 #7]
MSL B, [#0 #5]
ADD B, C 

SUB SP, #2
STR C, SP

; adds 2 to the flags and the
; the mod result will be here
ADD FLAGS, #2
SUB SP, #2
DIV A, #10
LDR C, SP
ADD C, #48
STB C, B 
ADD SP, #2
SUB FLAGS, #2

LDR C, SP
ADD SP, #2

ADD C, #1
JMP int_to_str_loop

reverse_buffer:
SUB SP, #2
STR C, SP
MOV C, #0   

reverse_buffer_loop:
LDR B, SP 
LDR A, SP 
DIV A, #2

GTE C, A
CJP dealloc_stack_and_return

; execute the swap at pos C 
; with pos B - 1 - C 
SUB SP, #1
MOV A, #0x0F
MSL A, [#0 #7]
MSL A, [#0 #5]
ADD A, C
; moves what is under A to stack 
CPY A, SP

; allocates 2 bytes on the stack
SUB SP, #2

; now stack holds the addr of A[C]
STR A, SP

; moves A to the other slice extreme 
SUB A, C
ADD A, B
SUB A, #1
SUB A, C 

; loads whats in the stack to B
LDR B, SP 

; given that B holds an address, moves
; what is under A to be under B
CPY A, B 

; reduce the stack, now it holds the char
; and not an address 
ADD SP, #2 

; moves what is the stack to be under A
CPY SP, A

; reduce the stack the amount we have allocated
ADD SP, #1

ADD C, #1
JMP reverse_buffer_loop

dealloc_stack_and_return:
LDR C, SP

MOV B, #0x0F
MSL B, [#0 #7]
MSL B, [#0 #5]
ADD B, C

MOV C, #3
STB C, B

ADD SP, #2

