// ==UserScript==
// @name        Xana Scraper RJS
// @namespace   xana.sh
// @match       *://xana.sh/*
// @grant       GM_xmlhttpRequest
// @version     1.0
// @author      -
// @description 10/1/2025, 3:13:37 PM
// @run-at      document-end
// ==/UserScript==

console.log("init scraper")

const socket = new WebSocket("ws://desk.f.xana.sh:8080");
socket.addEventListener("open", (event) => {
    socket.send(encodeOp(OP_INIT, document.location));
});
socket.addEventListener("message", (event) => {
    // console.log("raw request: " + event.data);
    let [op, data] = split_once(event.data, "\0");
    console.log(`received op ${op} data ${data}`);
    switch (op) {
        case OP_DEBUG:
            console.log("debug", data);
            break;
        case OP_SCRAPE:
            console.log("scrape", data);
            scrape(data);
            break;
        default:
            console.log("unknown op", op);
    }
});
socket.addEventListener("close", (event) => {
    console.log("Connection closed", event);
});
socket.addEventListener("error", (event) => {
    console.log(`ws error`, event);
})

//

const OP_INIT = "init";
const OP_DEBUG = "debug";
const OP_SCRAPE = "scrape";
const OP_CONTENT = "content";

function encodeOp(op, data) {
    return op + "\0" + data;
}

//

function scrape(url) {
    GM_xmlhttpRequest({
        url,
        responseType: "blob",
        onload: (e) => {
            socket.send(encodeOp(OP_CONTENT, e.status + "\0" + e.responseHeaders));
            socket.send(e.response);
        },
        onerror: (e) => {
            console.log("scrape error", e);
        },
    });
}

//

/** stupid javascript */
function split_once(string, separator) {
    let pos = string.indexOf(separator);
    return [
        string.substring(0, pos),
        string.substring(pos + 1),
    ]
}
