// One-time setup: create the public "images" bucket in Supabase Storage.
// Idempotent — running twice is safe (createBucket returns "already exists").
import { createClient } from '@supabase/supabase-js';
import { config } from 'dotenv';
config();

const supabase = createClient(process.env.SUPABASE_URL, process.env.SUPABASE_SERVICE_ROLE_KEY);

const BUCKET = 'images';

const { data: existing, error: listErr } = await supabase.storage.listBuckets();
if (listErr) {
  console.error('listBuckets:', listErr);
  process.exit(1);
}

const found = existing?.find((b) => b.name === BUCKET);
if (found) {
  console.log(`✓ bucket "${BUCKET}" already exists (public=${found.public})`);
  if (!found.public) {
    const { error: upErr } = await supabase.storage.updateBucket(BUCKET, { public: true });
    if (upErr) {
      console.error('updateBucket:', upErr);
      process.exit(1);
    }
    console.log(`  -> upgraded to public`);
  }
} else {
  const { error } = await supabase.storage.createBucket(BUCKET, {
    public: true,
    fileSizeLimit: 10 * 1024 * 1024, // 10MB hard cap server-side
    allowedMimeTypes: ['image/jpeg', 'image/png', 'image/webp', 'image/gif', 'image/avif'],
  });
  if (error) {
    console.error('createBucket:', error);
    process.exit(1);
  }
  console.log(`✓ created bucket "${BUCKET}" (public)`);
}

// Quick verify upload + delete round-trip — use a 1×1 PNG to satisfy the
// allowedMimeTypes filter on the bucket.
const PNG_1X1 = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
  'base64',
);
const testKey = `_health/${Date.now()}.png`;
const { error: uErr } = await supabase.storage
  .from(BUCKET)
  .upload(testKey, PNG_1X1, { contentType: 'image/png' });
if (uErr) {
  console.error('upload test:', uErr);
  process.exit(1);
}
const { data: urlData } = supabase.storage.from(BUCKET).getPublicUrl(testKey);
console.log(`✓ public URL pattern: ${urlData.publicUrl}`);
await supabase.storage.from(BUCKET).remove([testKey]);
console.log('✓ cleanup OK');
