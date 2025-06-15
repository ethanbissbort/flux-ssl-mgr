#!/bin/bash

# SSL Certificate Generation Script
# This script automates the process of generating SSL certificates using an intermediate CA
# 
# Usage:
#   Interactive mode: ./generate-ssl-cert.sh
#   Non-interactive mode: ./generate-ssl-cert.sh <hostname> <ip-address>
#   Help: ./generate-ssl-cert.sh -h|--help
#
# Examples:
#   ./generate-ssl-cert.sh web01 192.168.1.100
#   ./generate-ssl-cert.sh api-server 10.0.1.50

set -e  # Exit on any error

# Function to validate IP address format
validate_ip() {
    local ip=$1
    if [[ $ip =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]; then
        IFS='.' read -ra ADDR <<< "$ip"
        for i in "${ADDR[@]}"; do
            if [[ $i -gt 255 ]]; then
                return 1
            fi
        done
        return 0
    else
        return 1
    fi
}

# Function to validate hostname format
validate_hostname() {
    local hostname=$1
    if [[ $hostname =~ ^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?$ ]]; then
        return 0
    else
        return 1
    fi
}

# Function to show usage information
show_usage() {
    echo "SSL Certificate Generation Script"
    echo
    echo "Usage:"
    echo "  Interactive mode:     $0"
    echo "  Non-interactive mode: $0 <hostname> <ip-address>"
    echo "  Help:                 $0 -h|--help"
    echo
    echo "Arguments:"
    echo "  hostname     - The hostname identifier (will create <hostname>.fluxlab.systems)"
    echo "  ip-address   - The IP address for Subject Alternative Name"
    echo
    echo "Examples:"
    echo "  $0 web01 192.168.1.100"
    echo "  $0 api-server 10.0.1.50"
    echo "  $0 database 172.16.0.10"
    echo
    exit 1
}

echo "=== SSL Certificate Generation Script ==="
echo

# Check for help flag
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    show_usage
fi

# Determine mode based on arguments
if [[ $# -eq 0 ]]; then
    # Interactive mode
    echo "Running in interactive mode..."
    echo
    
    # Prompt for hostname/identifier
    while true; do
        read -p "Enter the hostname/identifier to replace TEMPLATE: " hostname
        if [[ -n "$hostname" ]] && validate_hostname "$hostname"; then
            break
        else
            echo "Error: Invalid hostname format. Please use alphanumeric characters and hyphens only."
        fi
    done

    # Prompt for IP address
    while true; do
        read -p "Enter the IP address for the Subject Alternative Name: " ip_address
        if validate_ip "$ip_address"; then
            break
        else
            echo "Error: Invalid IP address format. Please enter a valid IPv4 address (e.g., 192.168.1.1)"
        fi
    done
    
elif [[ $# -eq 2 ]]; then
    # Non-interactive mode
    hostname="$1"
    ip_address="$2"
    
    echo "Running in non-interactive mode..."
    echo
    
    # Validate hostname
    if [[ -z "$hostname" ]] || ! validate_hostname "$hostname"; then
        echo "Error: Invalid hostname format '$hostname'. Please use alphanumeric characters and hyphens only."
        exit 1
    fi
    
    # Validate IP address
    if ! validate_ip "$ip_address"; then
        echo "Error: Invalid IP address format '$ip_address'. Please enter a valid IPv4 address."
        exit 1
    fi
    
else
    echo "Error: Invalid number of arguments."
    echo
    show_usage
fi

# Construct the full domain name
full_domain="${hostname}.fluxlab.systems"

echo
echo "=== Configuration Summary ==="
echo "Hostname: $hostname"
echo "Full domain: $full_domain"
echo "IP Address: $ip_address"
echo

# Only prompt for confirmation in interactive mode
if [[ $# -eq 0 ]]; then
    read -p "Proceed with certificate generation? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo "Certificate generation cancelled."
        exit 0
    fi
else
    echo "Proceeding with certificate generation..."
fi

echo
echo "=== Starting Certificate Generation ==="

# Change to CA directory
echo "Changing to CA directory..."
cd /root/ca

# List directory contents for verification
echo "Current directory contents:"
ls -al

echo
echo "Generating private key..."
# Generate private key
openssl genrsa -out "intermediate/private/${full_domain}.key.pem" 2048

# Set secure permissions on private key
echo "Setting permissions on private key..."
chmod 400 "intermediate/private/${full_domain}.key.pem"

echo "Creating certificate signing request..."
# Create certificate signing request with IP SAN
openssl req -config intermediate/openssl.conf \
    -addext "subjectAltName = IP:${ip_address}" \
    -key "intermediate/private/${full_domain}.key.pem" \
    -new -sha256 \
    -out "intermediate/csr/${full_domain}.csr.pem"

echo "Signing certificate..."
# Sign the certificate
openssl ca -config intermediate/openssl.conf \
    -extensions server_cert \
    -days 375 \
    -notext \
    -md sha256 \
    -in "intermediate/csr/${full_domain}.csr.pem" \
    -out "intermediate/certs/${full_domain}.cert.pem"

echo "Copying certificate to output directory..."
# Copy certificate to output directory
cp "intermediate/certs/${full_domain}.cert.pem" /var/ssl/out/

echo
echo "=== Certificate Generation Complete ==="
echo
echo "Files generated:"
echo "  Private Key: intermediate/private/${full_domain}.key.pem"
echo "  Certificate: intermediate/certs/${full_domain}.cert.pem"
echo "  Output Copy: /var/ssl/out/${full_domain}.cert.pem"
echo

echo "=== Private Key Content ==="
cat "intermediate/private/${full_domain}.key.pem"

echo
echo "=== Certificate Content ==="
cat "/var/ssl/out/${full_domain}.cert.pem"

echo
echo "Certificate generation completed successfully!"
echo "Remember to keep the private key secure and never share it."