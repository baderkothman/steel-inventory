export const LOGO_ACCEPT = "image/png,image/jpeg,image/webp";
export const MAX_LOGO_BYTES = 2 * 1024 * 1024;

export async function readLogoFile(file: File) {
  if (!LOGO_ACCEPT.split(",").includes(file.type)) {
    throw new Error("Choose a PNG, JPEG, or WebP image.");
  }
  if (file.size <= 0 || file.size > MAX_LOGO_BYTES) {
    throw new Error("Logo files must be no larger than 2 MB.");
  }
  const dataUrl = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("The logo file could not be read."));
    reader.readAsDataURL(file);
  });
  return {
    mime_type: file.type,
    base64_data: dataUrl.slice(dataUrl.indexOf(",") + 1)
  };
}
