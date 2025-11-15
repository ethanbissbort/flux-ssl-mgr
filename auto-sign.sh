#!/bin/bash

set -e  # Exit on any error

# Configuration
WORKING_DIR="/root/ca"
OUTPUT_DIR="/home/fluxadmin/ssl/pem-out"
CA_KEY_TEMP="/tmp/ca_key_unlocked_$$.pem"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Cleanup function to remove temporary CA key
cleanup() {
    if [ -f "$CA_KEY_TEMP" ]; then
        echo "Cleaning up temporary files..."
        rm -f "$CA_KEY_TEMP"
    fi
}

# Set trap to cleanup on exit
trap cleanup EXIT

echo -e "${GREEN}PKI Certificate Signing Script${NC}"
echo "=============================="

# Check if we're in the right directory
if [ ! -d "$WORKING_DIR" ]; then
    echo -e "${RED}Error: Working directory $WORKING_DIR does not exist${NC}"
    exit 1
fi

cd "$WORKING_DIR"

# Check if intermediate CA key exists and if it's password protected
CA_KEY_PATH="intermediate/private/intermediate.key.pem"
if [ ! -f "$CA_KEY_PATH" ]; then
    echo -e "${RED}Error: Intermediate CA key not found at $CA_KEY_PATH${NC}"
    exit 1
fi

# Test if the CA key is password protected
echo "Checking intermediate CA key..."
if openssl rsa -in "$CA_KEY_PATH" -noout -check >/dev/null 2>&1; then
    echo -e "${GREEN}CA key is not password protected${NC}"
    CA_KEY_TO_USE="$CA_KEY_PATH"
else
    echo -e "${YELLOW}CA key is password protected${NC}"
    echo "Please enter the intermediate CA private key password once:"
    
    # Create a temporary unencrypted version of the CA key for this session
    if openssl rsa -in "$CA_KEY_PATH" -out "$CA_KEY_TEMP" 2>/dev/null; then
        chmod 600 "$CA_KEY_TEMP"
        CA_KEY_TO_USE="$CA_KEY_TEMP"
        echo -e "${GREEN}✓ CA key unlocked for this session${NC}"
    else
        echo -e "${RED}Error: Failed to unlock CA key${NC}"
        exit 1
    fi
fi

# Function to create a temporary openssl config that uses our unlocked key
create_temp_config() {
    local TEMP_CONFIG="/tmp/openssl_temp_$$.cnf"
    
    # Copy the original config
    cp "intermediate/openssl.cnf" "$TEMP_CONFIG"
    
    # Update the private_key path in the config to use our unlocked key
    sed -i "s|private_key.*=.*|private_key = $CA_KEY_TO_USE|g" "$TEMP_CONFIG"
    
    echo "$TEMP_CONFIG"
}

TEMP_OPENSSL_CONFIG=$(create_temp_config)

# Update cleanup function to also remove temp config
cleanup() {
    if [ -f "$CA_KEY_TEMP" ]; then
        echo "Cleaning up temporary files..."
        rm -f "$CA_KEY_TEMP"
    fi
    if [ -f "$TEMP_OPENSSL_CONFIG" ]; then
        rm -f "$TEMP_OPENSSL_CONFIG"
    fi
}

# Function to process a single certificate
process_certificate() {
    local CERT_NAME="$1"
    local SANS="$2"
    local USE_PASSWORD="$3"
    local CSR_PATH="$4"
    
    echo ""
    echo -e "${YELLOW}Processing certificate: ${CERT_NAME}${NC}"
    echo "=================================="
    
    # Copy CSR to intermediate directory if provided
    if [ -n "$CSR_PATH" ] && [ -f "$CSR_PATH" ]; then
        echo "Copying CSR: $CSR_PATH"
        cp "$CSR_PATH" "intermediate/csr/"
    fi

    # Generate private key
    echo "Generating private key..."
    if [[ "$USE_PASSWORD" =~ ^[Yy]$ ]]; then
        openssl genrsa -aes256 -out "intermediate/private/${CERT_NAME}.key.pem" 4096
    else
        openssl genrsa -out "intermediate/private/${CERT_NAME}.key.pem" 4096
    fi

    # Set permissions on private key
    chmod 400 "intermediate/private/${CERT_NAME}.key.pem"
    echo -e "${GREEN}✓ Private key generated${NC}"

    # Generate CSR with SANs
    echo "Generating certificate signing request..."
    openssl req -config intermediate/openssl.cnf \
        -key "intermediate/private/${CERT_NAME}.key.pem" \
        -addext "subjectAltName = $SANS" \
        -copy_extensions copyall \
        -new -sha256 \
        -out "intermediate/csr/${CERT_NAME}.csr.pem"
    echo -e "${GREEN}✓ CSR generated${NC}"

    # Sign the certificate (using temporary config with unlocked CA key)
    echo "Signing certificate with intermediate CA..."
    openssl ca -config "$TEMP_OPENSSL_CONFIG" \
        -extensions server_cert \
        -days 375 \
        -notext \
        -md sha256 \
        -batch \
        -in "intermediate/csr/${CERT_NAME}.csr.pem" \
        -out "intermediate/certs/${CERT_NAME}.cert.pem"
    echo -e "${GREEN}✓ Certificate signed${NC}"

    # Convert PEM to CRT format
    echo "Converting to CRT format..."
    cp "intermediate/certs/${CERT_NAME}.cert.pem" "intermediate/certs/${CERT_NAME}.crt"
    echo -e "${GREEN}✓ CRT format created${NC}"

    # Create output directory if it doesn't exist
    mkdir -p "$OUTPUT_DIR"

    # Copy files to output directory
    echo "Copying certificates to output directory..."
    cp "intermediate/certs/${CERT_NAME}.cert.pem" "$OUTPUT_DIR/"
    cp "intermediate/certs/${CERT_NAME}.crt" "$OUTPUT_DIR/"
    cp "intermediate/private/${CERT_NAME}.key.pem" "$OUTPUT_DIR/"

    # Set ownership and permissions
    chown fluxadmin:root "$OUTPUT_DIR/${CERT_NAME}.cert.pem"
    chown fluxadmin:root "$OUTPUT_DIR/${CERT_NAME}.crt" 
    chown fluxadmin:root "$OUTPUT_DIR/${CERT_NAME}.key.pem"
    chmod 755 "$OUTPUT_DIR/${CERT_NAME}.cert.pem"
    chmod 755 "$OUTPUT_DIR/${CERT_NAME}.crt"
    chmod 755 "$OUTPUT_DIR/${CERT_NAME}.key.pem"

    echo -e "${GREEN}✓ Files copied and permissions set${NC}"
    echo -e "${GREEN}✓ Certificate ${CERT_NAME} completed successfully${NC}"
}

# Function to scan and list CSR files
list_csr_files() {
    local DIR="$1"
    find "$DIR" -name "*.csr" -type f | sort
}

# Function to get certificate name from CSR filename
get_cert_name_from_csr() {
    local CSR_FILE="$1"
    basename "$CSR_FILE" .csr
}

# Choose processing mode
echo ""
echo "Select processing mode:"
echo "1) Single certificate (interactive)"
echo "2) Batch process CSR files from directory"
echo ""
read -p "Choose mode (1-2): " MODE

case "$MODE" in
    1)
        # Single certificate mode (original functionality)
        # Get certificate name
        read -p "Enter certificate name (e.g., myservice): " CERT_NAME
        if [ -z "$CERT_NAME" ]; then
            echo -e "${RED}Error: Certificate name is required${NC}"
            exit 1
        fi

        # Check if CSR already exists
        CSR_PATH="/home/fluxadmin/ssl/${CERT_NAME}.csr"
        if [ ! -f "$CSR_PATH" ]; then
            echo -e "${YELLOW}Warning: CSR not found at $CSR_PATH${NC}"
            read -p "Continue anyway? (y/N): " CONTINUE
            if [[ ! "$CONTINUE" =~ ^[Yy]$ ]]; then
                exit 1
            fi
            CSR_PATH=""
        else
            echo -e "${GREEN}Found CSR: $CSR_PATH${NC}"
        fi

        # Get Subject Alternative Names
        echo ""
        echo "Enter Subject Alternative Names (DNS and IP addresses):"
        echo "Example: DNS:service.fluxlab.systems,DNS:service.local,IP:10.0.2.100"
        read -p "SANs: " SANS
        if [ -z "$SANS" ]; then
            echo -e "${RED}Error: Subject Alternative Names are required${NC}"
            exit 1
        fi

        # Ask about password protection for private key
        echo ""
        read -p "Password protect the private key? (y/N): " USE_PASSWORD

        echo ""
        echo -e "${YELLOW}Starting certificate generation...${NC}"
        
        process_certificate "$CERT_NAME" "$SANS" "$USE_PASSWORD" "$CSR_PATH"
        ;;
        
    2)
        # Batch processing mode
        read -p "Enter directory containing CSR files [/home/fluxadmin/ssl]: " CSR_DIR
        CSR_DIR="${CSR_DIR:-/home/fluxadmin/ssl}"
        
        if [ ! -d "$CSR_DIR" ]; then
            echo -e "${RED}Error: Directory $CSR_DIR does not exist${NC}"
            exit 1
        fi
        
        # Find all CSR files
        CSR_FILES=($(list_csr_files "$CSR_DIR"))
        
        if [ ${#CSR_FILES[@]} -eq 0 ]; then
            echo -e "${RED}No CSR files found in $CSR_DIR${NC}"
            exit 1
        fi
        
        echo ""
        echo -e "${GREEN}Found ${#CSR_FILES[@]} CSR files:${NC}"
        for i in "${!CSR_FILES[@]}"; do
            CERT_NAME=$(get_cert_name_from_csr "${CSR_FILES[$i]}")
            echo "$((i+1)). $CERT_NAME (${CSR_FILES[$i]})"
        done
        
        echo ""
        echo "Selection options:"
        echo "  - Enter specific numbers (e.g., 1,3,5)"
        echo "  - Enter range (e.g., 1-3)"
        echo "  - Enter 'all' to process all files"
        read -p "Select CSRs to process: " SELECTION
        
        # Parse selection
        SELECTED_INDICES=()
        if [ "$SELECTION" = "all" ]; then
            for i in "${!CSR_FILES[@]}"; do
                SELECTED_INDICES+=($i)
            done
        elif [[ "$SELECTION" =~ ^[0-9]+-[0-9]+$ ]]; then
            # Range selection
            START=$(echo "$SELECTION" | cut -d'-' -f1)
            END=$(echo "$SELECTION" | cut -d'-' -f2)
            for ((i=$((START-1)); i<$((END)); i++)); do
                if [ $i -ge 0 ] && [ $i -lt ${#CSR_FILES[@]} ]; then
                    SELECTED_INDICES+=($i)
                fi
            done
        else
            # Comma-separated selection
            IFS=',' read -ra NUMS <<< "$SELECTION"
            for num in "${NUMS[@]}"; do
                num=$(echo "$num" | tr -d ' ')
                if [[ "$num" =~ ^[0-9]+$ ]] && [ $((num-1)) -ge 0 ] && [ $((num-1)) -lt ${#CSR_FILES[@]} ]; then
                    SELECTED_INDICES+=($((num-1)))
                fi
            done
        fi
        
        if [ ${#SELECTED_INDICES[@]} -eq 0 ]; then
            echo -e "${RED}No valid selections made${NC}"
            exit 1
        fi
        
        echo ""
        echo -e "${GREEN}Will process ${#SELECTED_INDICES[@]} certificates${NC}"
        
        # Get common settings for batch processing
        echo ""
        echo "For batch processing, you can set common Subject Alternative Names"
        echo "or configure each certificate individually."
        read -p "Use common SANs for all certificates? (y/N): " USE_COMMON_SANS
        
        COMMON_SANS=""
        if [[ "$USE_COMMON_SANS" =~ ^[Yy]$ ]]; then
            echo "Enter common Subject Alternative Names:"
            echo "Example: DNS:*.fluxlab.systems,IP:10.0.2.100"
            read -p "Common SANs: " COMMON_SANS
        fi
        
        read -p "Password protect all private keys? (y/N): " USE_PASSWORD
        
        echo ""
        echo -e "${YELLOW}Starting batch processing...${NC}"
        
        # Process selected certificates
        PROCESSED=0
        FAILED=0
        
        for idx in "${SELECTED_INDICES[@]}"; do
            CSR_FILE="${CSR_FILES[$idx]}"
            CERT_NAME=$(get_cert_name_from_csr "$CSR_FILE")
            
            SANS="$COMMON_SANS"
            if [ -z "$COMMON_SANS" ]; then
                echo ""
                echo -e "${YELLOW}Configure SANs for certificate: ${CERT_NAME}${NC}"
                echo "Enter Subject Alternative Names:"
                read -p "SANs for $CERT_NAME: " SANS
                if [ -z "$SANS" ]; then
                    echo -e "${RED}Skipping $CERT_NAME (no SANs provided)${NC}"
                    ((FAILED++))
                    continue
                fi
            fi
            
            if process_certificate "$CERT_NAME" "$SANS" "$USE_PASSWORD" "$CSR_FILE"; then
                ((PROCESSED++))
            else
                echo -e "${RED}Failed to process $CERT_NAME${NC}"
                ((FAILED++))
            fi
        done
        
        echo ""
        echo -e "${GREEN}Batch processing complete!${NC}"
        echo "Processed: $PROCESSED certificates"
        if [ $FAILED -gt 0 ]; then
            echo -e "${RED}Failed: $FAILED certificates${NC}"
        fi
        ;;
        
    *)
        echo -e "${RED}Invalid selection${NC}"
        exit 1
        ;;
esac

# Show summary based on processing mode
if [ "$MODE" = "1" ]; then
    echo ""
    echo -e "${GREEN}Certificate generation complete!${NC}"
    echo "Generated files:"
    echo "  • Certificate (PEM): $OUTPUT_DIR/${CERT_NAME}.cert.pem"
    echo "  • Certificate (CRT): $OUTPUT_DIR/${CERT_NAME}.crt"
    echo "  • Private Key:       $OUTPUT_DIR/${CERT_NAME}.key.pem"
    echo ""
    echo "Certificate details:"
    openssl x509 -in "intermediate/certs/${CERT_NAME}.cert.pem" -noout -text | grep -A1 "Subject Alternative Name" || echo "No SAN found"
    openssl x509 -in "intermediate/certs/${CERT_NAME}.cert.pem" -noout -subject -dates
fi

echo ""
echo -e "${YELLOW}Don't forget to update your service configuration with the new certificate(s)!${NC}"
