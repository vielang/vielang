// @ts-nocheck
const fs = require('fs');
const path = require('path');

const mapsDir = path.join(__dirname, '../public/game/maps');

// Map of image filenames to their keys used in BootScene
const imageToKey = {
  '2_1City_Terrains_32x32.png': '2_1City_Terrains_32x32',
  '3_1City_Props_32x32.png': '3_1City_Props_32x32',
  '4_1Generic_Buildings_32x32.png': '4_1Generic_Buildings_32x32',
  '5_1Floor_Modular_Buildings_32x32.png': '5_1Floor_Modular_Buildings_32x32',
  '5_2Floor_Modular_Buildings_32x32.png': '5_2Floor_Modular_Buildings_32x32',
  '5_3Floor_Modular_Buildings_32x32.png': '5_3Floor_Modular_Buildings_32x32',
  '5_5Floor_Modular_Buildings_32x32.png': '5_5Floor_Modular_Buildings_32x32',
  '5_6Floor_Modular_Buildings_32x32.png': '5_6Floor_Modular_Buildings_32x32',
  '5_Floor_Modular_Buildings_32x32.png': '5_Floor_Modular_Buildings_32x32',
  '16_1Office_32x32.png': '16_1Office_32x32',
  '1_2Terrains_and_Fences_32x32.png.png': '1_2Terrains_and_Fences_32x32.png',
  'vehicles.png': 'vehicles',
  'busStand.png': 'busStand',
  'tacoTruck.png': 'tacoTruck',
  'fancy_mat.png': 'fancy_mat',
  '1_Room_Builder_borders_32x32.png': '1_Room_Builder_borders_32x32',
  '2_LivingRoom_32x32.png': '2_LivingRoom_32x32',
  '3_1Bathroom_32x32.png.png': '3_1Bathroom_32x32.png',
  '3_2Bathroom_32x32.png.png': '3_2Bathroom_32x32.png',
  '4_1Bedroom_32x32.png': '4_1Bedroom_32x32',
  '5_Classroom_and_library_32x32.png': '5_Classroom_and_library_32x32',
  '13_Conference_Hall_32x32.png': '13_Conference_Hall_32x32',
  'Room_Builder_Floors_32x32.png': 'Room_Builder_Floors_32x32',
  'Room_Builder_Walls_32x32.png': 'Room_Builder_Walls_32x32',
};

// Fix tileset image paths in JSON files
function fixTilesetPaths(jsonFile) {
  const jsonPath = path.join(mapsDir, jsonFile);

  try {
    const data = JSON.parse(fs.readFileSync(jsonPath, 'utf8'));
    let modified = false;

    if (data.tilesets) {
      for (const tileset of data.tilesets) {
        if (tileset.image) {
          const filename = path.basename(tileset.image);
          const key = imageToKey[filename];

          if (key) {
            tileset.image = key;
            modified = true;
          }
        }
      }
    }

    if (modified) {
      fs.writeFileSync(jsonPath, JSON.stringify(data, null, 2));
      console.log(`✓ Fixed tileset paths in ${jsonFile}`);
    } else {
      console.log(`○ No changes needed for ${jsonFile}`);
    }
  } catch (err) {
    console.error(`✗ Error fixing ${jsonFile}:`, err.message);
  }
}

// Main execution
const jsonFiles = fs.readdirSync(mapsDir).filter(f => f.endsWith('.json'));
console.log(`Found ${jsonFiles.length} JSON files to fix...\n`);

jsonFiles.forEach(fixTilesetPaths);

console.log('\nPath fixing complete!');
