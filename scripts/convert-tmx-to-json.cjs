// @ts-nocheck
const fs = require('fs');
const path = require('path');
const xml2js = require('xml2js');

const mapsDir = path.join(__dirname, '../public/game/maps');
const parser = new xml2js.Parser();

// Get all TMX files
const tmxFiles = fs.readdirSync(mapsDir).filter(f => f.endsWith('.tmx'));

console.log(`Found ${tmxFiles.length} TMX files to convert...`);

tmxFiles.forEach(async (tmxFile) => {
  const tmxPath = path.join(mapsDir, tmxFile);
  const jsonPath = tmxPath.replace('.tmx', '.json');

  try {
    const xmlContent = fs.readFileSync(tmxPath, 'utf8');
    const result = await parser.parseStringPromise(xmlContent);

    // Convert TMX XML structure to Tiled JSON format
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

    // Convert tilesets
    if (map.tileset) {
      map.tileset.forEach((tileset) => {
        const tilesetData = {
          firstgid: parseInt(tileset.$.firstgid),
          source: tileset.$.source
        };
        mapData.tilesets.push(tilesetData);
      });
    }

    // Convert layers
    if (map.layer) {
      map.layer.forEach((layer) => {
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
      });
    }

    // Write JSON file
    fs.writeFileSync(jsonPath, JSON.stringify(mapData, null, 2));
    console.log(`✓ Converted ${tmxFile} -> ${path.basename(jsonPath)}`);
  } catch (err) {
    console.error(`✗ Error converting ${tmxFile}:`, err.message);
  }
});

console.log('\nConversion complete!');
