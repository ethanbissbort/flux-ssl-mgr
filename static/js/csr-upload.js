// CSR Upload Page JavaScript

document.addEventListener('DOMContentLoaded', function() {
    const form = document.getElementById('csr-upload-form');
    const fileInput = document.getElementById('csr-file');
    const dropZone = document.getElementById('csr-drop-zone');
    const uploadPlaceholder = dropZone.querySelector('.upload-placeholder');
    const fileInfo = document.getElementById('file-info');
    const removeFileBtn = document.getElementById('remove-file');
    const submitBtn = document.getElementById('submit-btn');
    const resultContainer = document.getElementById('result-container');
    const errorContainer = document.getElementById('error-container');

    let selectedFile = null;

    // File drag and drop
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
        dropZone.addEventListener(eventName, preventDefaults, false);
    });

    function preventDefaults(e) {
        e.preventDefault();
        e.stopPropagation();
    }

    ['dragenter', 'dragover'].forEach(eventName => {
        dropZone.addEventListener(eventName, () => {
            dropZone.classList.add('drag-over');
        });
    });

    ['dragleave', 'drop'].forEach(eventName => {
        dropZone.addEventListener(eventName, () => {
            dropZone.classList.remove('drag-over');
        });
    });

    dropZone.addEventListener('drop', handleDrop);

    function handleDrop(e) {
        const dt = e.dataTransfer;
        const files = dt.files;

        if (files.length > 0) {
            fileInput.files = files;
            handleFileSelect({ target: fileInput });
        }
    }

    // File selection
    fileInput.addEventListener('change', handleFileSelect);

    function handleFileSelect(e) {
        const file = e.target.files[0];
        if (file) {
            selectedFile = file;
            displayFileInfo(file);
        }
    }

    function displayFileInfo(file) {
        uploadPlaceholder.style.display = 'none';
        fileInfo.style.display = 'block';
        fileInfo.querySelector('.file-name').textContent = file.name;
        fileInfo.querySelector('.file-size').textContent = formatFileSize(file.size);
    }

    function formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
    }

    // Remove file
    removeFileBtn.addEventListener('click', function() {
        fileInput.value = '';
        selectedFile = null;
        uploadPlaceholder.style.display = 'block';
        fileInfo.style.display = 'none';
    });

    // Form submission
    form.addEventListener('submit', async function(e) {
        e.preventDefault();

        if (!selectedFile) {
            showError('Please select a CSR file');
            return;
        }

        const validityDays = document.getElementById('validity-days').value;

        // Show loading state
        const btnText = submitBtn.querySelector('.btn-text');
        const spinner = submitBtn.querySelector('.spinner');
        btnText.style.display = 'none';
        spinner.style.display = 'inline-block';
        submitBtn.disabled = true;

        const formData = new FormData();
        formData.append('csr_file', selectedFile);
        formData.append('validity_days', validityDays);

        try {
            const response = await fetch('/api/csr/upload', {
                method: 'POST',
                body: formData
            });

            const data = await response.json();

            if (response.ok && data.success) {
                showResult(data.certificate);
            } else {
                showError(data.error || 'Failed to sign certificate');
            }
        } catch (error) {
            showError('Network error: ' + error.message);
        } finally {
            btnText.style.display = 'inline';
            spinner.style.display = 'none';
            submitBtn.disabled = false;
        }
    });

    function showResult(certificate) {
        form.style.display = 'none';
        errorContainer.style.display = 'none';
        resultContainer.style.display = 'block';

        document.getElementById('cert-subject').textContent = certificate.subject;
        document.getElementById('cert-serial').textContent = certificate.serial;
        document.getElementById('cert-not-before').textContent = new Date(certificate.not_before).toLocaleString();
        document.getElementById('cert-not-after').textContent = new Date(certificate.not_after).toLocaleString();
        document.getElementById('cert-sans').textContent = certificate.sans.length > 0
            ? certificate.sans.join(', ')
            : 'None';
        document.getElementById('cert-pem').value = certificate.pem;
    }

    function showError(message) {
        form.style.display = 'none';
        resultContainer.style.display = 'none';
        errorContainer.style.display = 'block';
        document.getElementById('error-message').textContent = message;
    }

    // Download certificate
    document.getElementById('download-cert').addEventListener('click', function() {
        const pem = document.getElementById('cert-pem').value;
        const blob = new Blob([pem], { type: 'application/x-pem-file' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'certificate.pem';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    });

    // Copy to clipboard
    document.getElementById('copy-cert').addEventListener('click', async function() {
        const pem = document.getElementById('cert-pem').value;
        try {
            await navigator.clipboard.writeText(pem);
            this.textContent = 'Copied!';
            setTimeout(() => {
                this.textContent = 'Copy to Clipboard';
            }, 2000);
        } catch (err) {
            alert('Failed to copy to clipboard');
        }
    });

    // Sign another
    document.getElementById('sign-another').addEventListener('click', function() {
        form.style.display = 'block';
        resultContainer.style.display = 'none';
        errorContainer.style.display = 'none';
        form.reset();
        removeFileBtn.click();
    });

    // Try again
    document.getElementById('try-again').addEventListener('click', function() {
        form.style.display = 'block';
        errorContainer.style.display = 'none';
    });
});
