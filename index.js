const Router = require('koa-router'),
    koa = require('koa'),
    fs = require('fs'),
    config = require('./resources/config.js'),
    mongojs = require('mongojs'),
    db = mongojs(config.mongo.url, ['users']);

const app = new koa();
const route = new Router();

app.listen(config.server.port);
app.use(route.routes())
    .use(route.allowedMethods());

route.get('/get-user-location-records',  () => {
    let headerWritten = false;
    const fileName = `./${new Date().toISOString()}_User_Locations.csv`;
     db.users.find({}).forEach( (err, doc) =>{
        if(doc){
            if(!headerWritten){
                const headers = doc.locations[0].coords;
                const headersLine = Object.keys(headers).join(',');
                headersLine + ',timestamp,username';
                fs.appendFileSync(fileName, `${headersLine}\n`);
                headerWritten = true;
            }
            for (const eachLocation of doc.locations){
                const eachRow = Object.values(eachLocation.coords);
                eachRow.push(eachLocation.timestamp);
                eachRow.push(doc.username);
                fs.appendFileSync(fileName, `${eachRow.join(',')}\n`);
            }
        }
    })
});






