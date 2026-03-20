// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.bmp;

import java.awt.Image;
import java.nio.file.Path;
import java.util.List;
import java.util.Objects;

import javax.imageio.ImageIO;
import javax.swing.JOptionPane;

import org.infinity.gui.converter.AbstractConvertWorker;
import org.infinity.gui.converter.ConvertOutputPanel.Overwrite;
import org.infinity.gui.converter.WorkerResult;
import org.infinity.util.io.FileEx;
import org.infinity.util.io.FileManager;
import org.infinity.util.io.StreamUtils;
import org.tinylog.Logger;

/** Implementation of the {@link AbstractConvertWorker} for the BMP Converter. */
class BmpWorker extends AbstractConvertWorker<WorkerResult> {
  private final ConvertToBmp parent;
  private final List<Path> inputFiles;
  private final Path outputDir;
  private final Overwrite mode;
  private final boolean isAlpha;
  private final boolean isPremultipliedAlpha;
  private final boolean closeOnExit;

  /** Initializes the worker instance with all parameters needed for the conversion operation. */
  public BmpWorker(ConvertToBmp parent, List<Path> inputFiles, Path outputDir, Overwrite mode, boolean isAlpha,
      boolean isPremultipliedAlpha, boolean closeOnExit) {
    super(parent, true, true, 0, inputFiles.size(), "Converting to BMP...", 250, 1000);
    this.parent = Objects.requireNonNull(parent);
    this.inputFiles = Objects.requireNonNull(inputFiles);
    this.outputDir = Objects.requireNonNull(outputDir);
    this.mode = Objects.requireNonNull(mode);
    this.isAlpha = isAlpha;
    this.isPremultipliedAlpha = isPremultipliedAlpha;
    this.closeOnExit = closeOnExit;
    setProgressNote("Preparing");
  }

  @Override
  protected WorkerResult doInBackground() throws Exception {
    int failed = 0;
    int skipped = 0;
    for (int i = 0, count = inputFiles.size(); i < count; i++) {
      if (isProgressCancelled()) {
        cancel(false);
        return null;
      }

      setProgressNote("File " + i + " / " + count);
      advanceProgressTo(i);

      try {
        // preparing data
        final Path inFile = FileManager.resolve(inputFiles.get(i));
        final Path outFile = outputDir.resolve(StreamUtils.replaceFileExtension(inFile.getFileName().toString(), "BMP"));

        if (FileEx.create(outFile).exists()) {
          switch (mode) {
            case ASK:
            {
              final String msg = "File " + outFile + " already exists. Overwrite?";
              final int result = JOptionPane.showConfirmDialog(parent, msg, "Overwrite",
                  JOptionPane.YES_NO_CANCEL_OPTION, JOptionPane.QUESTION_MESSAGE);
              if (result == JOptionPane.NO_OPTION) {
                skipped++;
                continue;
              } else if (result == JOptionPane.CANCEL_OPTION) {
                cancel(false);
                return null;
              }
              break;
            }
            case SKIP:
              skipped++;
              continue;
            default:
          }
        }

        // converting to BMP
        Image image = ImageIO.read(inFile.toFile());
        if (image != null && outFile != null) {
          ConvertToBmp.writeBitmap(image, outFile, isAlpha, isPremultipliedAlpha);
        } else {
          throw new Exception("Unable to load source image: " + inFile);
        }
      } catch (Exception e) {
        failed++;
        Logger.debug(e);
      }
    }

    // creating summary message
    final String msg;
    final int skipCount = failed + skipped;
    if (skipCount > 0) {
      if (skipCount == 1) {
        msg = "1 input file has been skipped.";
      } else {
        msg = skipCount + " input files have been skipped.";
      }
    } else {
      msg = "Conversion finished successfully.";
    }

    return new WorkerResult(failed == 0, msg);
  }

  @Override
  protected void onCompleted(WorkerResult result) {
    final String msg = (result != null) ? result.getMessage() : "Conversion finished successfully.";
    if (result.isSuccess()) {
      JOptionPane.showMessageDialog(parent, msg, "Information", JOptionPane.INFORMATION_MESSAGE);
    } else {
      JOptionPane.showMessageDialog(parent, msg, "Error", JOptionPane.ERROR_MESSAGE);
    }
    parent.hideWindow(closeOnExit);
  }

  @Override
  protected void onCancelled() {
    JOptionPane.showMessageDialog(parent, "Conversion cancelled by the user.", "Error", JOptionPane.ERROR_MESSAGE);
    parent.hideWindow(closeOnExit);
  }
}