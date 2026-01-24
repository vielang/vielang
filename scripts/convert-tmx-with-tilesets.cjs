// @ts-nocheck
const fs = require('fs');
const path = require('path');
const xml2js = require('xml2js');

const mapsDir = path.join(__dirname, '../public/game/maps');
const tilesetsDir = path.join(mapsDir, 'tileSets');
const parser = new xml2js.Parser();

// Cache for loaded tilesets
const tilesetCache = {};

// Load and parse a TSX file
async function loadTileset(tsxPath) {
  const fullPath = path.join(mapsDir, tsxPath);

  if (tilesetCache[fullPath]) {
    return tilesetCache[fullPath];
  }

  try {
    const xmlContent = fs.readFileSync(fullPath, 'utf8');
    const result = await parser.parseStringPromise(xmlContent);
    const tileset = result.tileset;

    const tilesetData = {
      name: tileset.$.name,
      tilewidth: parseInt(tileset.$.tilewidth),
      tileheight: parseInt(tileset.$.tileheight),
      tilecount: parseInt(tileset.$.tilecount),
      columns: parseInt(tileset.$.columns),
      image: tileset.image[0].$.source.replace('../images/', ''),
      imagewidth: parseInt(tileset.image[0].$.width),
      imageheight: parseInt(tileset.image[0].$.height),
    };

    tilesetCache[fullPath] = tilesetData;
    return tilesetData;
  } catch (err) {
    console.error(`Error loading tileset ${tsxPath}:`, err.message);
    return null;
  }
}

// Convert TMX to JSON with embedded tilesets
async function convertTMX(tmxFile) {
  const tmxPath = path.join(mapsDir, tmxFile);
  const jsonPath = tmxPath.replace('.tmx', '.json');

  try {
    const xmlContent = fs.readFileSync(tmxPath, 'utf8');
    const result = await parser.parseStringPromise(xmlContent);
    const map = result.map;

    const mapData = {
      compressionlevel: -1,
      height: parseInt(map.$.height),
      width: parseInt(map.$.width),
      infinite: map.$.infinite === '1',
      layers: [],
      nextlayerid: parseInt(map.$.nextlayerid || 1),
      nextobjectid: parseInt(map.$.nextobjectid || 1),
      orientation: map.$.orientation || 'orthogonal',
      renderorder: map.$.renderorder || 'right-down',
      tiledversion: map.$.tiledversion,
      tileheight: parseInt(map.$.tileheight),
      tilewidth: parseInt(map.$.tilewidth),
      tilesets: [],
      type: 'map',
      version: map.$.version
    };

    // Load and embed tilesets
    if (map.tileset) {
      for (const tileset of map.tileset) {
        const firstgid = parseInt(tileset.$.firstgid);
        const source = tileset.$.source;

        const tilesetData = await loadTileset(source);
        if (tilesetData) {
          mapData.tilesets.push({
            firstgid,
            name: tilesetData.name,
            tilewidth: tilesetData.tilewidth,
            tileheight: tilesetData.tileheight,
            tilecount: tilesetData.tilecount,
            columns: tilesetData.columns,
            // Important: Phaser needs this field for embedded tilesets
            image: tilesetData.image,
            imagewidth: tilesetData.imagewidth,
            imageheight: tilesetData.imageheight,
          });
        }
      }
    }

    // Convert layers
    if (map.layer) {
      for (const layer of map.layer) {
        const layerData = {
          data: [],
          height: parseInt(layer.$.height),
          id: parseInt(layer.$.id),
          name: layer.$.name,
          opacity: 1,
          type: 'tilelayer',
          visible: true,
          width: parseInt(layer.$.width),
          x: 0,
          y: 0
        };

        // Parse layer data (base64 encoded)
        if (layer.data && layer.data[0]) {
          const encoding = layer.data[0].$.encoding;
          if (encoding === 'base64') {
            const base64Data = layer.data[0]._.trim();
            const buffer = Buffer.from(base64Data, 'base64');

            // Convert buffer to tile array (4 bytes per tile = uint32)
            for (let i = 0; i < buffer.length; i += 4) {
              const tileId = buffer.readUInt32LE(i);
              layerData.data.push(tileId);
            }
          }
        }

        mapData.layers.push(layerData);
      }
    }

    // Convert object layers
    if (map.objectgroup) {
      for (const objectGroup of map.objectgroup) {
        const objectLayerData = {
          id: parseInt(objectGroup.$.id),
          name: objectGroup.$.name,
          objects: [],
          opacity: 1,
          type: 'objectgroup',
          visible: true,
          x: 0,
          y: 0
        };

        if (objectGroup.object) {
          for (const obj of objectGroup.object) {
            const objData = {
              height: parseFloat(obj.$.height || 0),
              id: parseInt(obj.$.id),
              name: obj.$.name || '',
              rotation: parseFloat(obj.$.rotation || 0),
              type: obj.$.type || '',
              visible: true,
              width: parseFloat(obj.$.width || 0),
              x: parseFloat(obj.$.x),
              y: parseFloat(obj.$.y)
            };

            // Add properties if they exist
            if (obj.properties && obj.properties[0] && obj.properties[0].property) {
              objData.properties = {};
              for (const prop of obj.properties[0].property) {
                objData.properties[prop.$.name] = prop.$.value;
              }
            }

            objectLayerData.objects.push(objData);
          }
        }

        mapData.layers.push(objectLayerData);
      }
    }

    // Write JSON file
    fs.writeFileSync(jsonPath, JSON.stringify(mapData, null, 2));
    console.log(`✓ Converted ${tmxFile} -> ${path.basename(jsonPath)} (with ${mapData.tilesets.length} embedded tilesets)`);
  } catch (err) {
    console.error(`✗ Error converting ${tmxFile}:`, err.message);
  }
}

// Main execution
async function main() {
  const tmxFiles = fs.readdirSync(mapsDir).filter(f => f.endsWith('.tmx'));
  console.log(`Found ${tmxFiles.length} TMX files to convert...\n`);

  for (const tmxFile of tmxFiles) {
    await convertTMX(tmxFile);
  }

  console.log('\nConversion complete!');
}

main();
