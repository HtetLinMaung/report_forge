# Report Forge

## Overview

Report Forge is an innovative report generation microservice, leveraging the power of Rust and the 'sitetopdf' npm package. Designed for converting HTML content into well-formatted PDF reports, it's encapsulated in a Docker container for effortless deployment.

## Features

- `HTML to PDF Conversion`: Transforms HTML documents or web pages into PDFs.
- `Template-Based Reports`: Generates dynamic PDF reports from HTML templates.
- `PDF Retrieval`: Enables direct fetching of generated PDF reports.
- `Rust-Powered Performance`: Built using Rust for optimal performance and reliability.
- `Customizable PDF Output`: Offers diverse customization options for PDFs.
- `Docker Containerization`: Simplifies deployment and scalability.

## Installation

To install, ensure Docker is set up, then:

```bash
docker pull htetlinmaung/report_forge
docker run -d -p 8080:8080 htetlinmaung/report_forge
```

## API Usage

1. `/api/site-to-pdf`

Converts HTML or a web page into a PDF.

Request Parameters:

- `url` or `content`: Source for conversion.
- Customization options: `format`, `landscape`, `scale`, etc.

Example:

```bash
curl -X POST [API_ENDPOINT] -H "Content-Type: application/json" -d '{"url": "http://example.com"}'
```

2. `/api/process-report`

Processes an HTML template into a PDF report.

Request Parameters:

- `template_name`: The HTML template name.
- `options`: PDF customization options.

Example:

```bash
curl -X POST [API_ENDPOINT] -H "Content-Type: application/json" -d '{"template_name": "report_template", "options": {...}}'
```

3. `/reports/{file_name}`

Retrieves a generated PDF report.

Path Parameter:

- `file_name`: Name of the generated PDF file.

Example:

```bash
curl [API_ENDPOINT]/reports/{file_name}
```
