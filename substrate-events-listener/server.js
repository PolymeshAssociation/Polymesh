// Import the API and selected RxJs operators
const { switchMap } = require('rxjs/operators');
const { ApiRx } = require('@polkadot/api');
const dataService = require('./lib/dataService');
const utils = require('./lib/utils');

require('dotenv').config();

// code from https://polkadot.js.org/api/examples/rx/08_system_events/
async function main() {
  // get event filters from config
  const eventsFilter = utils.getEventSections();
  // initialize the data service
  // internally connects to all storage sinks
  await dataService.init();

  // Create API with connection to the local Substrate node
  // If your Substrate node is running elsewhere, add the config (server + port) in .env
  // Use the config in the create function below
  ApiRx.create()
    .pipe(
      switchMap((api) =>
        api.query.system.events()
      ))
    // subscribe to system events via storage
    .subscribe(async (events) => {
      events.forEach(async (record) => {
        // extract the event object
        const { event, phase } = record;
        // check section filter
        if (eventsFilter.includes(event.section.toString()) || eventsFilter.includes("all")) {
          // create event object for data sink
          const eventObj = {
            section: event.section,
            method: event.method,
            meta: event.meta.documentation.toString(),
            data: event.data.toString()
          }

          // remove this log if not needed
          console.log('Event Received: ' + Date.now() + ": " + JSON.stringify(eventObj));

          // insert in data sink
          // can have some error handling here
          // should not fail or catch exceptions gracefully
          await dataService.insert(eventObj);
        }
      });
    });
};

main().catch((error) => {
  console.error(error);
  process.exit(-1);
});