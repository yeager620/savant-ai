use anyhow::Result;
use image::{DynamicImage, ImageBuffer, Luma, GenericImageView};
use imageproc::contrast::adaptive_threshold;
use imageproc::filter::{gaussian_blur_f32, median_filter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingConfig {
    pub enabled: bool,
    pub denoise: bool,
    pub enhance_contrast: bool,
    pub adaptive_threshold: bool,
    pub gaussian_blur: Option<f32>,
    pub scale_factor: Option<f32>,
    pub dpi_target: Option<u32>,
}

impl Default for PreprocessingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            denoise: true,
            enhance_contrast: true,
            adaptive_threshold: true,
            gaussian_blur: Some(0.5),
            scale_factor: None,
            dpi_target: Some(300), // Target DPI for OCR
        }
    }
}

pub struct ImagePreprocessor {
    config: PreprocessingConfig,
}

impl ImagePreprocessor {
    pub fn new(config: PreprocessingConfig) -> Self {
        Self { config }
    }

    pub fn process(&self, image: &DynamicImage) -> Result<DynamicImage> {
        if !self.config.enabled {
            return Ok(image.clone());
        }

        let mut processed = image.clone();
        
        // Resize very large images to prevent memory issues and improve performance
        let (width, height) = processed.dimensions();
        if width * height > 4_000_000 {
            let scale_factor = (4_000_000.0 / (width * height) as f64).sqrt();
            tracing::info!("Resizing large image ({}x{}) by factor {:.2}", width, height, scale_factor);
            processed = self.scale_image(&processed, scale_factor as f32)?;
            let (new_width, new_height) = processed.dimensions();
            tracing::info!("Image resized to {}x{}", new_width, new_height);
        }

        // Convert to grayscale for better OCR performance
        processed = DynamicImage::ImageLuma8(processed.to_luma8());

        // Scale image if needed for optimal OCR DPI
        if let Some(scale_factor) = self.config.scale_factor {
            processed = self.scale_image(&processed, scale_factor)?;
        } else if let Some(target_dpi) = self.config.dpi_target {
            processed = self.scale_to_dpi(&processed, target_dpi)?;
        }

        // Apply Gaussian blur for noise reduction
        if let Some(sigma) = self.config.gaussian_blur {
            processed = self.apply_gaussian_blur(&processed, sigma)?;
        }

        // Denoise using median filter
        if self.config.denoise {
            processed = self.denoise(&processed)?;
        }

        // Enhance contrast using adaptive thresholding
        if self.config.adaptive_threshold {
            processed = self.apply_adaptive_threshold(&processed)?;
        }

        // Final contrast enhancement
        if self.config.enhance_contrast {
            processed = self.enhance_contrast(&processed)?;
        }

        Ok(processed)
    }

    fn scale_image(&self, image: &DynamicImage, scale_factor: f32) -> Result<DynamicImage> {
        let (width, height) = image.dimensions();
        let new_width = (width as f32 * scale_factor) as u32;
        let new_height = (height as f32 * scale_factor) as u32;
        
        Ok(image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3))
    }

    fn scale_to_dpi(&self, image: &DynamicImage, target_dpi: u32) -> Result<DynamicImage> {
        // Assume current image is 72 DPI (common screen resolution)
        let current_dpi = 72;
        let scale_factor = target_dpi as f32 / current_dpi as f32;
        
        // Only scale up, not down (to avoid losing information)
        if scale_factor > 1.0 {
            self.scale_image(image, scale_factor)
        } else {
            Ok(image.clone())
        }
    }

    fn apply_gaussian_blur(&self, image: &DynamicImage, sigma: f32) -> Result<DynamicImage> {
        match image {
            DynamicImage::ImageLuma8(img) => {
                let blurred = gaussian_blur_f32(img, sigma);
                Ok(DynamicImage::ImageLuma8(blurred))
            }
            _ => {
                // Convert to grayscale first
                let gray = image.to_luma8();
                let blurred = gaussian_blur_f32(&gray, sigma);
                Ok(DynamicImage::ImageLuma8(blurred))
            }
        }
    }

    fn denoise(&self, image: &DynamicImage) -> Result<DynamicImage> {
        match image {
            DynamicImage::ImageLuma8(img) => {
                let denoised = median_filter(img, 1, 1);
                Ok(DynamicImage::ImageLuma8(denoised))
            }
            _ => {
                let gray = image.to_luma8();
                let denoised = median_filter(&gray, 1, 1);
                Ok(DynamicImage::ImageLuma8(denoised))
            }
        }
    }

    fn apply_adaptive_threshold(&self, image: &DynamicImage) -> Result<DynamicImage> {
        let (width, height) = image.dimensions();
        
        // Skip adaptive threshold for very large images to prevent integer overflow
        // in imageproc's integral image calculation
        if width * height > 2_000_000 {
            tracing::warn!("Image too large for adaptive threshold ({}x{}), using simple threshold", width, height);
            return self.apply_simple_threshold(image);
        }
        
        match image {
            DynamicImage::ImageLuma8(img) => {
                let thresholded = adaptive_threshold(img, 15);
                Ok(DynamicImage::ImageLuma8(thresholded))
            }
            _ => {
                let gray = image.to_luma8();
                let thresholded = adaptive_threshold(&gray, 15);
                Ok(DynamicImage::ImageLuma8(thresholded))
            }
        }
    }
    
    fn apply_simple_threshold(&self, image: &DynamicImage) -> Result<DynamicImage> {
        let gray = image.to_luma8();
        let mut result = gray.clone();
        
        // Apply simple binary threshold (Otsu-like approach)
        let mut histogram = [0u32; 256];
        for pixel in gray.pixels() {
            histogram[pixel[0] as usize] += 1;
        }
        
        // Find optimal threshold using Otsu's method approximation
        let total_pixels = (gray.width() * gray.height()) as f64;
        let mut sum_total = 0f64;
        for i in 0..256 {
            sum_total += (i as f64) * (histogram[i] as f64);
        }
        
        let mut sum_background = 0f64;
        let mut weight_background = 0f64;
        let mut max_variance = 0f64;
        let mut optimal_threshold = 0u8;
        
        for threshold in 0..256 {
            weight_background += histogram[threshold] as f64;
            if weight_background == 0.0 { continue; }
            
            let weight_foreground = total_pixels - weight_background;
            if weight_foreground == 0.0 { break; }
            
            sum_background += (threshold as f64) * (histogram[threshold] as f64);
            let mean_background = sum_background / weight_background;
            let mean_foreground = (sum_total - sum_background) / weight_foreground;
            
            let between_class_variance = weight_background * weight_foreground * 
                (mean_background - mean_foreground).powi(2);
            
            if between_class_variance > max_variance {
                max_variance = between_class_variance;
                optimal_threshold = threshold as u8;
            }
        }
        
        // Apply threshold
        for pixel in result.pixels_mut() {
            pixel[0] = if pixel[0] > optimal_threshold { 255 } else { 0 };
        }
        
        Ok(DynamicImage::ImageLuma8(result))
    }

    fn enhance_contrast(&self, image: &DynamicImage) -> Result<DynamicImage> {
        match image {
            DynamicImage::ImageLuma8(img) => {
                let enhanced = self.apply_contrast_enhancement(img)?;
                Ok(DynamicImage::ImageLuma8(enhanced))
            }
            _ => {
                let gray = image.to_luma8();
                let enhanced = self.apply_contrast_enhancement(&gray)?;
                Ok(DynamicImage::ImageLuma8(enhanced))
            }
        }
    }

    fn apply_contrast_enhancement(&self, image: &ImageBuffer<Luma<u8>, Vec<u8>>) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>> {
        let (width, height) = image.dimensions();
        let mut enhanced = ImageBuffer::new(width, height);

        // Calculate histogram
        let mut histogram = [0u32; 256];
        for pixel in image.pixels() {
            histogram[pixel[0] as usize] += 1;
        }

        // Calculate cumulative distribution function
        let mut cdf = [0u32; 256];
        cdf[0] = histogram[0];
        for i in 1..256 {
            cdf[i] = cdf[i - 1] + histogram[i];
        }

        let total_pixels = (width * height) as f32;
        
        // Apply histogram equalization
        for (x, y, pixel) in enhanced.enumerate_pixels_mut() {
            let old_value = image.get_pixel(x, y)[0] as usize;
            let new_value = ((cdf[old_value] as f32 / total_pixels) * 255.0) as u8;
            *pixel = Luma([new_value]);
        }

        Ok(enhanced)
    }
}