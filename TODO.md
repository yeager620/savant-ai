TODO:
I) finish "smart database" for long-term memort of screen capture data
|-> automatic processing pipeline of screen capture data from .png format to smart database with meaningful data (e.g. all text with location on screen formatted in such a way so that sentences can be distinguished and seperated from one another) using machine learning approaches or whatever is most effective and efficient
|-> mcp server for smart database that will allow chatbots to interact with it and query with natural language / text (e.g. what did I work on July 2nd 2025?, how much coding did I do today?, what websites did I visit yesterday?, what are all the names of the people i met with over google meets today?, etc.)
|-> *Note: final functionality for the project as a whole might include automatic detection of a coding problem on-screen, then sending the processed problem as a prompt to a chatbot/LLM and automatically generating the LLM's solution to the problem in a GUI in real time; the implementation of the project's daemons, data pipeline(s), and databases should allow for this fucntionality in eventuality while still adering to the project's UNIX-like philosophy and design principles 
II) Improved GUI
|-> Keep theme & styling in line with current design meant to look similar to the MacOS "pro" terminal profile theme
|-> Dropdown menu which extends from the MacOS menu bar icon when clicked (top of the screen, the app icon / logo sits here) for selecting tool(s)/options: 1. toggle GUI (this toggles the visibility of the GUI which sits on the left side of the screen by default right now) 2. start/stop daemons (toggles activity of the audio and screen capture daemons) 3. see app analytics (shows how much space in bytes/KB/MB/GB the application's databases for audio and visual data are taking up, how much RAM the application is consuming, and any other useful relevant metrics)
III) Improved chatbot(s) and LLM API
|-> More Ollama model options (add llama3.2 for now and make it the default instead of devstral)
|-> unified chatbot interface for querying both audio and visual data via the MCP servers as opposed to the strictly audio data chatbot that exists currently (keep vanilla chatbot option seperate)
IV) figure out method for containerization and/or automated release workflow
V) figure out how to replicate functionality of this project on non-macOS systems, i.e. windows & linux
VI) unit tests for audio and screen capture data pipeline with consistent / given data (e.g. deciphering, transforming, and storing computer generated audio & screenshots with known content that can be verified)


