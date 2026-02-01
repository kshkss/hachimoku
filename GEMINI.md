# Internal Implementation Policies

- **Go-based Backend:** The entire backend is written in the Go programming language.
- **Server-Side Templating:** HTML pages are generated on the server using the `templ` library.
- **Embedded Database:** Utilizes an embedded database, simplifying the setup by not requiring an external database server.
- **Data Integrity:** While not strictly enforcing all accounting rules, it guarantees that debits and credits for each transaction are always balanced.
- **Authentication:** User authentication is handled using the `goth` library.
- **Modern Interface:** The user interface is built with the Bulma CSS framework, featuring dynamic elements controlled by Alpine.js and seamless data updates via htmx.
