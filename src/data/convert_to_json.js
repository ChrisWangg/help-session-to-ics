const fs = require('fs');
const path = require('path');
const { hours, names } = require('./export.js');

// Convert the `names` object to a JSON string with pretty formatting
const namesJson = JSON.stringify(names, null, 2);

// Convert the `hours` array (allocations) to a JSON string with pretty formatting
const allocationsJson = JSON.stringify(hours, null, 2);

// Define output paths
const namesOutputPath = path.join(__dirname, 'tutors.json');
const allocationsOutputPath = path.join(__dirname, 'allocations.json');

// Write the JSON strings to their respective files
fs.writeFileSync(namesOutputPath, namesJson, 'utf8');
fs.writeFileSync(allocationsOutputPath, allocationsJson, 'utf8');

console.log('Data has been split and saved to names.json and allocations.json');
