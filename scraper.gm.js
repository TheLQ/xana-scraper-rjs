// ==UserScript==
// @name        test script
// @namespace   Violentmonkey Scripts
// @match       *://xana.sh/*
// @grant GM_download
// @version     1.0
// @author      -
// @description 10/1/2025, 3:13:37 PM
// @run-at document-end
// ==/UserScript==

console.log("hello")
GM_download({
    url: "https://",
    name: "600369.torrent",
    onload: (e) => {
        console.log("hi onload");
        console.log("r", e)
    }
})

