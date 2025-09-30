ADD FLAGS, #2

MOV A, #104
MOV B, #44

CALL min_max

init_loop:
; condicao do loop
MUL B, A

; ler max da stack
ADD SP, #2
LDR A, SP
SUB SP, #2

; ler min da stack
LDR C, SP

loop:
GT A, B
CJP end

SUB SP, #2
STR A, SP

; calcula modulo
SUB SP, #2
DIV A, C

; le o modulo da stack em A
LDR A, SP

; libera a stack
ADD SP, #2

; se o modulo for 0 
; e só sair
EQ A, #0
LDR A, SP
CJP end

STR B, SP
ADD SP, #4
LDR B, SP
SUB SP, #4

ADD A, B
LDR B, SP

ADD SP, #2
JMP loop

end:

DBG

ADD SP, #6

CALL int_to_str

DBG

; Mostrar numero no terminal
MOV C, #2
MOV B, #0x0F
MSL B, [#1 #4]
MSL B, [#1 #7]
MSL B, [#0 #1]
STR C, B

ADD FLAGS, #1

; ====== FUNCAO MIN_MAX ======
; esta funcao armazena na stack
; em ordem decrescente os valores
; do REG A e do REG B

min_max:
GT A, B
CJP store_a_b
JMP store_b_a

store_a_b:
SUB SP, #2
STR A, SP

SUB SP, #2
STR B, SP
RET

store_b_a:
SUB SP, #2
STR B, SP

SUB SP, #2
STR A, SP
RET

; ====== FUNCAO PRINT ======
; esta funcao mostra o MMC
; no terminal

int_to_str:

GT A, #9
CJP int_to_str_bef_loop

MOV B, #0x0F
MSL B, [#0 #7]
MSL B, [#0 #5]
ADD A, #48
STB A, B
ADD B, #1

; indicates end_of_text
MOV A, #3
STB A, B

RET

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
SUB SP, #2
DIV A, #10
LDR C, SP
ADD C, #48
STB C, B 
ADD SP, #2

LDR C, SP
ADD SP, #2

ADD C, #1
JMP int_to_str_loop

reverse_buffer:
DBG

SUB SP, #2
STR C, SP
MOV C, #0   

reverse_buffer_loop:
LDR B, SP
LDR A, SP

SUB SP, #2
DIV A, #2
ADD SP, #2

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

DBG

; moves what is the stack to be under A
CPY SP, A

; reduce the stack the amount we have allocated
ADD SP, #1

ADD C, #1
JMP reverse_buffer_loop

dealloc_stack_and_return:
LDR C, SP

DBG

MOV B, #0x0F
MSL B, [#0 #7]
MSL B, [#0 #5]
ADD B, C

MOV C, #3
STB C, B

ADD SP, #2
RET






