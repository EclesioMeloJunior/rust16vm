MOV A, #100
MOV B, #16

LOOP:
EQ B, #0
CJP END

ADD FLAGS #2
SUB SP, #2
DIV A, B
LDR C, SP
ADD SP, #2
SUB FLAGS, #2

MOV A, B 
MOV B, C
JMP LOOP


END:
MOV C, #0x0F
MSL C, [#0 #7]
MSL C, [#0 #5]

GT A, #9
CJP LOOP_terminal

ADD A, #48
STR A, C
ADD C, #1
MOV M, #3
STR M, C


MOV A, #2
MOV C, #0x0F
MSL C, [#1 #4]
MSL C, [#1 #7]
MSL C, [#0 #1]
STR A, C
JMP END_terminal


LOOP_terminal:
MOV M, A
ADD FLAGS, #2
SUB SP, #2
DIV M, #10
LDR M, SP
ADD SP, #2
SUB FLAGS, #2

DIV A, #10
ADD A, #48
ADD C, #1
STR A, C

ADD M, #48
ADD C, #1
STR M, C


MOV A, #2
MOV C, #0x0F
MSL C, [#1 #4]
MSL C, [#1 #7]
MSL C, [#0 #1]
STR A, C



END_terminal:

ADD FLAGS #1
