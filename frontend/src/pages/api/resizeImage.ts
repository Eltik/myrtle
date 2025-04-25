import type { NextApiRequest, NextApiResponse } from 'next';
import sharp from 'sharp';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { url, width, height, atlasUrl } = req.query;

  if (!url || typeof url !== 'string') {
    return res.status(400).json({ error: 'URL is required' });
  }

  try {
    // Fetch the image
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch image: ${response.statusText}`);
    }

    const buffer = await response.arrayBuffer();

    // Process the image with sharp
    let sharpInstance = sharp(Buffer.from(buffer));

    // If atlasUrl is provided, fetch and parse it to get the actual expected dimensions
    if (atlasUrl && typeof atlasUrl === 'string') {
      try {
        const atlasResponse = await fetch(atlasUrl);
        if (atlasResponse.ok) {
          const atlasText = await atlasResponse.text();
          
          // Look for frame definitions in the atlas file
          // Example: "name: region_0\n  rotate: false\n  xy: 0, 0\n  size: 440, 440\n  orig: 440, 440\n  offset: 0, 0\n  index: -1"
          const frameRegex = /size: (\d+), (\d+)/g;
          let match;
          let maxWidth = 0;
          let maxHeight = 0;
          
          while ((match = frameRegex.exec(atlasText)) !== null) {
            const [_, frameWidth, frameHeight] = match;
            maxWidth = Math.max(maxWidth, parseInt(frameWidth!, 10));
            maxHeight = Math.max(maxHeight, parseInt(frameHeight!, 10));
          }
          
          // If we found frame dimensions, use them instead of the size field
          if (maxWidth > 0 && maxHeight > 0) {
            console.log(`Using frame dimensions from atlas: ${maxWidth}x${maxHeight}`);
            sharpInstance = sharpInstance.resize(maxWidth, maxHeight, {
              fit: 'fill',
              withoutEnlargement: false,
            });
          } else if (width && height && !isNaN(Number(width)) && !isNaN(Number(height))) {
            // Fall back to the size field if no frame dimensions found
            sharpInstance = sharpInstance.resize(Number(width), Number(height), {
              fit: 'fill',
              withoutEnlargement: false,
            });
          }
        }
      } catch (atlasError) {
        console.error('Error parsing atlas file:', atlasError);
        // Fall back to width/height if atlas parsing fails
        if (width && height && !isNaN(Number(width)) && !isNaN(Number(height))) {
          sharpInstance = sharpInstance.resize(Number(width), Number(height), {
            fit: 'fill',
            withoutEnlargement: false,
          });
        }
      }
    } else if (width && height && !isNaN(Number(width)) && !isNaN(Number(height))) {
      // If no atlas URL provided, use the width and height parameters
      sharpInstance = sharpInstance.resize(Number(width), Number(height), {
        fit: 'fill',
        withoutEnlargement: false,
      });
    }

    // Convert to PNG format
    const processedBuffer = await sharpInstance.png().toBuffer();

    // Set appropriate headers
    res.setHeader('Content-Type', 'image/png');
    res.setHeader('Cache-Control', 'public, max-age=31536000, immutable');
    
    // Send the processed image
    res.send(processedBuffer);
  } catch (error) {
    console.error('Error processing image:', error);
    res.status(500).json({ error: 'Failed to process image' });
  }
} 