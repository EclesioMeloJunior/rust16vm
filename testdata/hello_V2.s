MOV A, #72
MOV B, #0x0F
MSL B, [#1 #10]
STR A, B

MOV A, #101
ADD B, #1
STR A, B
    
MOV A, #108
ADD B, #1
STR A, B                                                                        
MOV A, #108
ADD B, #1
STR A, B

MOV A, #111
ADD B, #1
STR A, B

MOV A, #2
MOV B, #0x0F
MSL B, [#0 #5]
STR A, B
