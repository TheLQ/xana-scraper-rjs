// ==UserScript==
// @name        test script
// @namespace   Violentmonkey Scripts
// @match       *://xana.sh/*
// @grant       GM_xmlhttpRequest
// @version     1.0
// @author      -
// @description 10/1/2025, 3:13:37 PM
// @run-at document-end
// ==/UserScript==

console.log("init scraper")

// Create WebSocket connection.
const socket = new WebSocket("ws://desk.f.xana.sh:8080");
socket.addEventListener("open", (event) => {
    socket.send(encodeOp(OP_INIT, document.location));
});
socket.addEventListener("message", (event) => {
    // console.log("raw request: " + event.data);
    let [op, data] = split_once(event.data, ":");
    console.log(`received op ${op} data ${data}`);
    switch (op) {
        case OP_MESSAGE:
            console.log("message", data);
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
const OP_MESSAGE = "message";
const OP_SCRAPE = "scrape";
const OP_CONTENT = "content";

function encodeOp(op, data) {
    return op + ":" + data;
}

//

function scrape(url) {
    GM_xmlhttpRequest({
        url,
        responseType: "blob",
        onload: (e) => {
            socket.send(encodeOp(OP_CONTENT, e.status + ":" + e.responseHeaders));
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

// class EncodeOp {
//     #op;
//
//     constructor(op) {
//         this.op = op;
//     }
//
//     encode(data) {
//         return this.#op + ":" + data;
//     }
// }
//
// const INIT_OP = new EncodeOp("init");
// const OP = new EncodeOp("start_page");