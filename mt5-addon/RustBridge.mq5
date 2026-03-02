//+------------------------------------------------------------------+
//|                                                   RustBridge.mq5 |
//|                                     Copyright 2026, prop-bots    |
//|                                             https://www.mql5.com |
//+------------------------------------------------------------------+
#property copyright "prop-bots"
#property link      "https://github.com/prop-bots/prop-bots"
#property version   "1.00"

// We use the built-in socket functions of MQL5 to run a simple TCP server
// listening for the Rust engine to connect.
input int InputPort = 5556;

int server_socket = INVALID_HANDLE;
int client_socket = INVALID_HANDLE;

//+------------------------------------------------------------------+
//| Expert initialization function                                   |
//+------------------------------------------------------------------+
int OnInit()
  {
   Print("Initializing RustBridge EA...");
   server_socket = SocketCreate();
   if(server_socket == INVALID_HANDLE)
     {
      Print("Failed to create socket, Error: ", GetLastError());
      return(INIT_FAILED);
     }
     
   if(!SocketBind(server_socket, InputPort))
     {
      Print("Failed to bind socket on port ", InputPort, " Error: ", GetLastError());
      SocketClose(server_socket);
      return(INIT_FAILED);
     }

   if(!SocketListen(server_socket, 1))
     {
      Print("Failed to listen on socket, Error: ", GetLastError());
      SocketClose(server_socket);
      return(INIT_FAILED);
     }

   Print("RustBridge EA listening on port ", InputPort);
   
   // Create a timer to poll for incoming connections
   EventSetMillisecondTimer(100);

   return(INIT_SUCCEEDED);
  }

//+------------------------------------------------------------------+
//| Expert deinitialization function                                 |
//+------------------------------------------------------------------+
void OnDeinit(const int reason)
  {
   EventKillTimer();
   if(client_socket != INVALID_HANDLE)
     {
      SocketClose(client_socket);
     }
   if(server_socket != INVALID_HANDLE)
     {
      SocketClose(server_socket);
     }
   Print("RustBridge EA stopped.");
  }

//+------------------------------------------------------------------+
//| Timer function for polling sockets                               |
//+------------------------------------------------------------------+
void OnTimer()
  {
   // Check for new connection if none exists
   if(client_socket == INVALID_HANDLE)
     {
      client_socket = SocketAccept(server_socket, 10); // 10ms timeout
      if(client_socket != INVALID_HANDLE)
        {
         Print("Client connected!");
         // Send connected message
         string msg = "{\"type\":\"connected\",\"version\":\"1.0.0\"}";
         SendMessage(msg);
        }
     }
   else
     {
      // Check for incoming data
      uint len = SocketIsReadable(client_socket);
      if(len > 0)
        {
         uchar buffer[];
         if(SocketRead(client_socket, buffer, len, 10) > 0)
           {
            // Process the raw bytes
            // In a real implementation this would assemble length-prefixed chunks
            // For now we just print
            string json = CharArrayToString(buffer);
            Print("Received: ", json);
            // Example response: ACK the keepalive/heartbeat
            if(StringFind(json, "heartbeat") >= 0)
              {
               SendMessage("{\"type\":\"heartbeat_ack\",\"timestamp\":\"" + TimeToString(TimeCurrent(), TIME_DATE|TIME_SECONDS) + "\"}");
              }
           }
        }
      else
        {
         // Check if disconnected
         if(!SocketIsConnected(client_socket))
           {
            Print("Client disconnected.");
            SocketClose(client_socket);
            client_socket = INVALID_HANDLE;
           }
        }
     }
  }

//+------------------------------------------------------------------+
//| Expert tick function                                             |
//+------------------------------------------------------------------+
void OnTick()
  {
   // Here you would broadcast ticks/bars to the client if subscribed
   if(client_socket != INVALID_HANDLE)
     {
      // Example bare minimum tick broadcast
      // SendMessage("{\"type\":\"tick\",\"instrument\":\"" + Symbol() + "\" ... }")
     }
  }

//+------------------------------------------------------------------+
//| Helper to frame and send a JSON string                           |
//+------------------------------------------------------------------+
bool SendMessage(string json)
  {
   if(client_socket == INVALID_HANDLE) return false;
   
   uchar body[];
   StringToCharArray(json, body, 0, StringLen(json));
   
   uint len = ArraySize(body);
   uchar header[4];
   
   // 4-byte big-endian length prefix
   header[0] = (uchar)((len >> 24) & 0xFF);
   header[1] = (uchar)((len >> 16) & 0xFF);
   header[2] = (uchar)((len >> 8) & 0xFF);
   header[3] = (uchar)(len & 0xFF);
   
   uchar frame[];
   ArrayCopy(frame, header, 0, 0, 4);
   ArrayCopy(frame, body, 4, 0, len);
   
   int sent = SocketSend(client_socket, frame, ArraySize(frame));
   return (sent > 0);
  }
//+------------------------------------------------------------------+
