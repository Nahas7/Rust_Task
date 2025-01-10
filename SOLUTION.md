# Solution

Here you can document all bugs and design flaws.

Report : 
################################################################################################################## 
01- Bug 	 : The buf found in handle function Client::handle
				it's assumed that the received message has a size that will be fit in one call 
				, if the message size is larger than this , the remaining bytes will not be received.
	 
	Solution : 	Use loop to repeat the receiving until all the message bytes are received by using message data vector to accumulate  the 
				incoming data

02-	Bug 	 : in Client::handle function , the function handles the EchoMessage only and doesn't handle the AddRquest
	Solution : Handle both EchoMessage and AddRequest , and use "match" statement fro message handling to handle
		       different types of messages	
			   
		
03- Bug 	 : start server listening for incoming clients in a while loop will lead to CPU load 
	Solution : Enter the stream in non-blocking mode and If there are no incoming connections, 
			   sleep for 100 milliseconds to avoid busy-waiting. 

04- Bug 	 : The system handles each client connection in a single thread.
	Solution : For each incoming client a new thread is spawned to handle this client.
################################################################################################################## 


# test cases :
All test cases Passed 
