MOV A, #0

loop:
EQ A, #10
CJP end

MOV C, #72

MOV B, #0x0F
MSL B, [#0 #7]
MSL B, [#0 #5]
ADD B, A

STR C, B

ADD A, #1
JMP loop

end:
MOV C, #2
MOV B, #0x0F
MSL B, [#1 #4]
MSL B, [#1 #7]
MSL B, [#0 #1]
STR C, B

ADD FLAGS, #1
