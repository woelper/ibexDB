# ibexDB

ibexDB is a very simple key value database that can be accessed via webbrowser.




curl, POST:
```bash
curl -d '{"key":"name","value":"zaphod"}' -H "Content-Type: application/json" -X POST http://localhost:8000/add
```