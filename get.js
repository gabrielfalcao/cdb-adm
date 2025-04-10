#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const target = process.argv[2] == undefined ? process.argv[1] : process.argv[2];


const bytes = fs.readFileSync(target);
const string = Array.from(bytes)
            .map(c => String.fromCharCode(c))
            .join("");
const data = JSON.parse(string);


console.log(data["com.apple.gms.availability.disallowedUseCases"].map(c => String.fromCharCode(c)).join(""))
