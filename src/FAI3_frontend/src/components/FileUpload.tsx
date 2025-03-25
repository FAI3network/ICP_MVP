import { ChangeEvent, useRef, useState } from "react";
import { Input } from "./ui";

export default function FileUpload({ onFileChange, accept="*/*", multiple=false, showFileName=true }: { accept?: string, onFileChange: (file: File | File[]) => void, multiple?: boolean, showFileName?: boolean }) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [fileName, setFileName] = useState<string | null>(null);

  const handleFileSelect = (e: ChangeEvent<HTMLInputElement>) => {
    if (multiple) {
      const selectedFiles = e.target.files;
      if (selectedFiles) {
        const files = Array.from(selectedFiles);
        const fileNames = files.map((file) => file.name).join(", ");
        setFileName(fileNames);
        onFileChange(files)
      }
    } else {
      const selectedFile = e.target.files?.item(0);
      if (selectedFile) {
        setFileName(selectedFile.name);
        onFileChange(selectedFile);
      }
    }
  }

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
    fileInputRef.current?.click();
  }

  const handleDragOver = (event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
  };
  
  const handleFileDrop = (event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    if (event.dataTransfer.files && event.dataTransfer.files[0]) {
      const file = event.dataTransfer.files[0];
      setFileName(file.name);
      onFileChange(file);
    }
  };

  return (
    <div 
      className="flex flex-col items-center justify-center w-full h-full border-2 border-dashed border-gray-300 rounded-lg my-4"
    >
      <div
        onDragOver={handleDragOver}
        onDrop={handleFileDrop}
        onClick={handleClick}
        className="flex flex-col items-center justify-center w-full h-full py-6 cursor-pointer"
      >
        {fileName && showFileName ? (
          <span>{fileName}</span>
        ) : (
          <span>Drag and Drop here </span>
        )}
        <input
          ref={fileInputRef}
          type="file"
          accept={accept}
          style={{ display: "none" }}
          onChange={handleFileSelect}
          multiple={multiple}
        />
      </div>
  </div>
  );
}