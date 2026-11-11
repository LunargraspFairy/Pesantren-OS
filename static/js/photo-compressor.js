async function compressMeetingPhoto(file, maxWidth = 1280, quality = 0.75) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    const objectUrl = URL.createObjectURL(file);

    img.onload = () => {
      URL.revokeObjectURL(objectUrl);
      let width = img.width;
      let height = img.height;

      if (width > maxWidth) {
        height = Math.round((height * maxWidth) / width);
        width = maxWidth;
      }

      const canvas = document.createElement('canvas');
      canvas.width = width;
      canvas.height = height;

      const ctx = canvas.getContext('2d');
      ctx.drawImage(img, 0, 0, width, height);

      canvas.toBlob(
        (blob) => {
          if (!blob) return reject(new Error('Gagal mengompres foto pertemuan'));
          const compressedFile = new File([blob], 'meeting_logbook.webp', {
            type: 'image/webp',
            lastModified: Date.now(),
          });
          resolve(compressedFile);
        },
        'image/webp',
        quality
      );
    };

    img.onerror = (err) => {
      URL.revokeObjectURL(objectUrl);
      reject(err);
    };

    img.src = objectUrl;
  });
}

async function submitCompressedLogbook(event, visitId) {
  event.preventDefault();
  const form = event.target;
  const fileInput = form.querySelector('input[type="file"]');
  const notesInput = form.querySelector('textarea[name="meeting_notes"]');
  const submitBtn = form.querySelector('button[type="submit"]');

  if (!fileInput || !fileInput.files || fileInput.files.length === 0) {
    alert('Silakan ambil atau pilih foto pertemuan Wali Santri & Santri terlebih dahulu.');
    return;
  }

  try {
    submitBtn.disabled = true;
    submitBtn.textContent = 'Mengompres & Mengunggah...';

    const rawFile = fileInput.files[0];
    const webpFile = await compressMeetingPhoto(rawFile, 1280, 0.75);

    const formData = new FormData();
    formData.append('photo', webpFile, 'meeting_logbook.webp');
    if (notesInput && notesInput.value.trim() !== '') {
      formData.append('meeting_notes', notesInput.value.trim());
    }

    const response = await fetch(`/api/visits/${visitId}/logbook`, {
      method: 'POST',
      body: formData,
      credentials: 'same-origin',
    });

    if (!response.ok) {
      const errData = await response.json().catch(() => ({}));
      throw new Error(errData.message || `Gagal mengunggah logbook (HTTP ${response.status})`);
    }

    window.location.reload();
  } catch (err) {
    alert(err.message || 'Terjadi kesalahan saat menyimpan logbook');
    submitBtn.disabled = false;
    submitBtn.textContent = 'Simpan Logbook & Selesaikan';
  }
}
