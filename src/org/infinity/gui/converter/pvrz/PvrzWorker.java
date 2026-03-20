// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.pvrz;

import java.nio.file.Path;
import java.util.List;
import java.util.Objects;

import javax.swing.JOptionPane;

import org.infinity.gui.converter.AbstractConvertWorker;
import org.infinity.gui.converter.ConvertOutputPanel.Overwrite;
import org.infinity.gui.converter.WorkerResult;
import org.infinity.util.io.FileEx;
import org.infinity.util.io.StreamUtils;
import org.infinity.util.tuples.Triple;
import org.tinylog.Logger;

/** Implementation of the {@link AbstractConvertWorker} for the PVRZ Converter. */
class PvrzWorker extends AbstractConvertWorker<WorkerResult> {
  private final ConvertToPvrz parent;
  private final List<Path> inputFiles;
  private final Path outputDir;
  private final Overwrite mode;
  private final CompressionType compressionType;
  private final boolean closeOnExit;

  public PvrzWorker(ConvertToPvrz parent, List<Path> inputFiles, Path outputDir, Overwrite mode, CompressionType type,
      boolean closeOnExit) {
    super(parent, true, true, 0, inputFiles.size(), "Converting to PVRZ...", 250, 1000);
    this.parent = Objects.requireNonNull(parent);
    this.inputFiles = Objects.requireNonNull(inputFiles);
    this.outputDir = Objects.requireNonNull(outputDir);
    this.mode = Objects.requireNonNull(mode);
    this.compressionType = Objects.requireNonNull(type);
    this.closeOnExit = closeOnExit;
    setProgressNote("Preparing");
  }

  @Override
  protected WorkerResult doInBackground() throws Exception {
    final String msgSuccess = "Conversion finished successfully.";

    int warnings = 0;
    int errors = 0;
    int skipped = 0;
    for (int i = 0, count = inputFiles.size(); i < count; i++) {
      if (isProgressCancelled()) {
        cancel(false);
        return null;
      }

      setProgressNote("File " + i + " / " + count);
      advanceProgressTo(i);

      // preparations
      final Path inFile = inputFiles.get(i);
      final boolean isGraphics = ConvertToPvrz.isValidGraphicsInput(inFile);
      final boolean isPVR = ConvertToPvrz.isValidPvrInput(inFile);
      if (!isGraphics && !isPVR) {
        warnings++;
        skipped++;
        continue;
      }

      try {
        final Path outFile = outputDir.resolve(StreamUtils.replaceFileExtension(inFile.getFileName().toString(), "PVRZ"));
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

        // loading source image
        if (isPVR) {
          ConvertToPvrz.compressPvr(inFile, outFile);
        } else {
          final Triple<Boolean, Integer, Integer> result = ConvertToPvrz.encodePvrz(inFile, outFile, compressionType);
          if (!result.getValue0()) {
            skipped++;
          }
          warnings += result.getValue1();
          errors += result.getValue2();
        }
      } catch (Exception e) {
        errors++;
        Logger.debug(e);
      }
    }

    // creating summary message
    final StringBuilder sb = new StringBuilder();
    if (warnings == 0 && errors == 0) {
      sb.append(msgSuccess);
    } else {
      sb.append("Conversion finished with ");
      if (warnings > 0) {
        sb.append(warnings).append(" warning(s)");
      }
      if (errors > 0) {
        if (warnings > 0) {
          sb.append(" and ");
        }
        sb.append(errors).append(" errors");
      }
      sb.append('.');
    }

    if (skipped == 1) {
      sb.append("\n1 file has been skipped.");
    } else if (skipped > 1) {
      sb.append('\n').append(skipped).append(" files have been skipped.");
    }

    return new WorkerResult(errors + warnings == 0, sb.toString());
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