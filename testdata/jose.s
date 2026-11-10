MOV A, #83
MOV B, #0x0F
MSL B, [#0 #6]
MSL B, [#0 #6]
STR A, B

MOV A, #98
ADD B, #1
STR A, B
    
MOV A, #97
ADD B, #1
STR A, B

MOV A, #114
ADD B, #1
STR A, B

MOV A, #97
ADD B, #1
STR A, B

MOV A, #105
ADD B, #1
STR A, B

MOV A, #110
ADD B, #1
STR A, B

MOV A, #105
ADD B, #1
STR A, B


MOV A, #2
MOV B, #0x0F
MSL B, [#1 #4]
MSL B, [#1 #7]
MSL B, [#0 #1]
STR A, B


ADD FLAGS, #1