const dbName = 'file-cache';
const store = 'file-cache';

function openDB() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(dbName, 1);

    request.onupgradeneeded = () => {
      const db = request.result;

      if (db.objectStoreNames.contains(store)) db.deleteObjectStore(store);
      db.createObjectStore(store);
    };

    request.onerror = () => reject();
    request.onsuccess = () => resolve(request.result);
  });
}

function getFile(db, name) {
  return new Promise((resolve, reject) => {
    const req = db.transaction([store]).objectStore(store).get(name);

    req.onerror = () => reject();
    req.onsuccess = () => (req.result) ? resolve(req.result) : reject();
  });
}

function storeFile(db, file, name) {
  return openDB()
    .then(db => new Promise((resolve, reject) => {
      const req = db.transaction([store], 'readwrite')
                    .objectStore(store)
                    .put(file, name);

      req.onerror = (err) => reject(err);
      req.onsuccess = () => (req.result) ? resolve() : reject();
    }))
    .then(() => file);
}

function tryFetchFile(name) {
  // try to get file from ./assets
  let url = `../assets/${name}`;
  // add file extension if not present
  if (!~name.indexOf('.')) url += '.z3';

  return fetch(url).then((response) => {
    if (response.ok) return response.arrayBuffer();
    throw new Error(`Error loading file @ ${url}`, url);
  });
}

function saveFile(name, file) {
  return openDB().then(db => storeFile(db, file, name));
}

function loadFile(name) {
  return openDB()
    .then(
      db => getFile(db, name).catch(
        () => tryFetchFile(name).then(file => storeFile(db, file, name))
      ),
      () => tryFetchFile(name)
    );
}

function getKeys() {
  return openDB().then(db => new Promise((resolve, reject) => {
    const req = db.transaction([store]).objectStore(store).getAllKeys();

    req.onerror = () => reject();
    req.onsuccess = () => (req.result) ? resolve(req.result) : reject();
  }));
}

export default {
  save: saveFile,
  load: loadFile,
  keys: getKeys,
};
