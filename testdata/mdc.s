MOV A, #40
MOV B, #96
ADD FLAGS, #2
DBG
LT A, B
CJP if
MOV C, A
MOV A, B
MOV A, C

if:
MOV C, #0x0F
MSL C, [#0 #6]
MSL C, [#0 #6]

loop1:
SUB SP, #2
DIV B, A
MOV B, A
LDR A, SP
ADD SP, #2
NEQ A, #0
CJP loop1


MOV C #0
loop2:
SUB SP, #2
DIV B #10
LDR A, SP
MUL C, #10
ADD C, A
NEQ B, #0
CJP loop2

MOV B, C

MOV C, #0x0F
MSL C, [#0 #6]
MSL C, [#0 #6]

loop3:
SUB SP, #2
DIV B #10
LDR A, SP
ADD A, #48
STR A, C
ADD SP, #2
ADD C, #1
NEQ B, #0
CJP loop3


MOV A, #2
MOV B, #0x0F
MSL B, [#1 #4]
MSL B, [#1 #7]
MSL B, [#0 #1]
STR A, B


ADD FLAGS, #1