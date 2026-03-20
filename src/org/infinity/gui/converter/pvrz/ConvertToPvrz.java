// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.pvrz;

import java.awt.Dimension;
import java.awt.Graphics2D;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.image.BufferedImage;
import java.awt.image.DataBufferInt;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.nio.channels.SeekableByteChannel;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import javax.imageio.ImageIO;
import javax.swing.JOptionPane;
import javax.swing.JPanel;
import javax.swing.filechooser.FileNameExtensionFilter;

import org.infinity.gui.ChildFrame;
import org.infinity.gui.ViewerUtil;
import org.infinity.gui.converter.ConvertButtonsPanel;
import org.infinity.gui.converter.ConvertFileListPanel;
import org.infinity.gui.converter.ConvertOutputPanel;
import org.infinity.gui.converter.ConvertOutputPanel.Overwrite;
import org.infinity.gui.converter.PanelUpdateEvent;
import org.infinity.gui.converter.PanelUpdateListener;
import org.infinity.icon.Icons;
import org.infinity.resource.Profile;
import org.infinity.resource.graphics.ColorConvert;
import org.infinity.resource.graphics.Compressor;
import org.infinity.resource.graphics.DxtEncoder;
import org.infinity.util.DynamicArray;
import org.infinity.util.io.FileEx;
import org.infinity.util.io.FileManager;
import org.infinity.util.io.StreamUtils;
import org.infinity.util.tuples.Triple;
import org.tinylog.Logger;

/**
 * Dialog for converting graphics files into the PVRZ format.
 */
public class ConvertToPvrz extends ChildFrame implements PanelUpdateListener {
  private static Path currentPath = Profile.getGameRoot();

  private ConvertFileListPanel fileListPanel;
  private ConvertOutputPanel outputPanel;
  private PvrzOptionsPanel optionsPanel;
  private ConvertButtonsPanel buttonsPanel;

  public ConvertToPvrz() {
    super("Convert to PVRZ", true);
    init();
  }

  // --------------------- Begin Class ChildFrame ---------------------

  /** Resets dialog content and optionally hides the dialog window. */
  public void hideWindow(boolean hide) {
    reset();
    if (hide) {
      setVisible(false);
    }
  }

  @Override
  protected boolean windowClosing(boolean forced) throws Exception {
    currentPath = outputPanel.getCurrentDirectory();
    reset();
    return super.windowClosing(forced);
  }

  // --------------------- End Class ChildFrame ---------------------

  // --------------------- Begin Interface PanelUpdateListener ---------------------

  @Override
  public void panelUpdated(PanelUpdateEvent e) {
    if (e.getSource() == fileListPanel) {
      handleFileListPanel(e);
    } else if (e.getSource() == outputPanel) {
      handleOutputPanel(e);
    } else if (e.getSource() == buttonsPanel) {
      handleButtonPanel(e);
    }
  }

  // --------------------- End Interface PanelUpdateListener ---------------------

  /** Handles state changes in the file list panel. */
  private void handleFileListPanel(PanelUpdateEvent e) {
    updateStatus();
  }

  /** Handles state changes in the output panel. */
  private void handleOutputPanel(PanelUpdateEvent e) {
    updateStatus();
  }

  /** Handles state changes in the button panel. */
  private void handleButtonPanel(PanelUpdateEvent e) {
    if (e.getReason() == ConvertButtonsPanel.REASON_CONVERT) {
      convert();
    } else if (e.getReason() == ConvertButtonsPanel.REASON_CANCEL) {
      hideWindow(true);
    }
  }

  /** Resets session-specific dialog controls. */
  private void reset() {
    fileListPanel.reset();
    outputPanel.reset();
    optionsPanel.reset();
  }

  /** Updates the dialog control state. */
  private void updateStatus() {
    final int inputFileCount = fileListPanel.getFileCount();
    final boolean outputPathSet = !outputPanel.getOutputFolder().isEmpty();
    buttonsPanel.setConvertButtonEnabled(inputFileCount > 0 && outputPathSet);
  }

  /** Performs the file conversion. */
  private void convert() {
    final List<Path> inFiles = fileListPanel.getFiles();
    if (inFiles.isEmpty()) {
      JOptionPane.showMessageDialog(this, "No input files defined.", "Error", JOptionPane.ERROR_MESSAGE);
      return;
    }

    Path outPath = null;
    try {
      outPath = FileManager.resolve(outputPanel.getOutputFolder());
    } catch (Exception e) {
      Logger.debug(e);
    }

    if (outPath == null) {
      JOptionPane.showMessageDialog(this, "Invalid output directory specified.", "Error", JOptionPane.ERROR_MESSAGE);
      return;
    }

    final Overwrite mode = outputPanel.getOverwriteMode();
    final CompressionType type = optionsPanel.getCompressionType();
    final boolean closeOnExit = buttonsPanel.isCloseOnExit();

    try {
      final PvrzWorker worker = new PvrzWorker(this, inFiles, outPath, mode, type, closeOnExit);
      worker.execute();
    } catch (Exception e) {
      Logger.error(e);
      final String msg;
      if (e.getMessage() == null || e.getMessage().isEmpty()) {
        msg = "Error: " + e.getMessage();
      } else {
        msg = e.getClass().getSimpleName() + " error occurred.";
      }
      JOptionPane.showMessageDialog(this, msg, "Error", JOptionPane.ERROR_MESSAGE);
    }
  }

  private void init() {
    setIconImage(Icons.ICON_APPLICATION_16.getIcon().getImage());

    fileListPanel = new ConvertFileListPanel("Input", currentPath, getFileNameExtensionFilters());
    fileListPanel.addPanelUpdateListener(this);

    optionsPanel = new PvrzOptionsPanel();

    outputPanel = new ConvertOutputPanel("Output", currentPath, optionsPanel);
    outputPanel.addPanelUpdateListener(this);

    buttonsPanel = new ConvertButtonsPanel();
    buttonsPanel.addPanelUpdateListener(this);

    final GridBagConstraints c = new GridBagConstraints();

    final JPanel panelMain = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.BOTH,
        new Insets(0, 0, 0, 0), 0, 0);
    panelMain.add(fileListPanel, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelMain.add(outputPanel, c);
    ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelMain.add(buttonsPanel, c);

    setLayout(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 1.0, GridBagConstraints.CENTER, GridBagConstraints.BOTH,
        new Insets(8, 8, 8, 8), 0, 0);
    add(panelMain, c);

    updateStatus();

    setMinimumSize(getPreferredSize());
    pack();
    setLocationRelativeTo(getParent());
    setVisible(true);
  }

  /** Returns a list of supported input file extension filters. */
  public static List<FileNameExtensionFilter> getFileNameExtensionFilters() {
    final List<FileNameExtensionFilter> retVal = new ArrayList<>(ColorConvert.GRAPHICS_FILTERS_DEFAULT.size() + 1);
    retVal.addAll(ColorConvert.GRAPHICS_FILTERS_DEFAULT);
    retVal.add(new FileNameExtensionFilter("PVR files (*.pvr)", "pvr"));
  
    // generating updated catch-all filter
    final List<String> extensions = new ArrayList<>();
    for (int i = 1, count = ColorConvert.GRAPHICS_FILTERS_DEFAULT.size(); i < count; i++) {
      final FileNameExtensionFilter filter = ColorConvert.GRAPHICS_FILTERS_DEFAULT.get(i);
      extensions.addAll(Arrays.asList(filter.getExtensions()));
    }
  
    final StringBuilder sb = new StringBuilder();
    sb.append("Graphics files (");
    for (int i = 0; i < extensions.size(); i++) {
      if (i > 0) {
        sb.append(", ");
      }
      sb.append("*.").append(extensions.get(i));
    }
    sb.append(')');
  
    retVal.set(0, new FileNameExtensionFilter(sb.toString(), extensions.toArray(new String[extensions.size()])));
  
    return retVal;
  }

  /** Checks graphics input file properties. */
  public static boolean isValidGraphicsInput(Path inFile) {
    boolean result = (inFile != null && FileEx.create(inFile).isFile());
    if (result) {
      Dimension d = ColorConvert.getImageDimension(inFile);
      if (d.width <= 0 || d.width > 1024 || d.height <= 0 || d.height > 1024) {
        result = false;
      }
    }
    return result;
  }

  /** Checks PVR input file properties. */
  public static boolean isValidPvrInput(Path inFile) {
    boolean result = (inFile != null && FileEx.create(inFile).isFile());
    if (result) {
      try (final InputStream is = StreamUtils.getInputStream(inFile)) {
        final String sig = StreamUtils.readString(is, 4);
        if ("PVR\u0003".equals(sig)) {
          StreamUtils.readInt(is); // flags
          StreamUtils.readInt(is); // pixel format #1
          StreamUtils.readInt(is); // pixel format #2
          StreamUtils.readInt(is); // color space
          StreamUtils.readInt(is); // channel type
          final int h = StreamUtils.readInt(is); // height
          final int w = StreamUtils.readInt(is); // width
          if (h <= 0 || w <= 0 || h > 1024 || w > 1024) {
            result = false;
          }
        } else {
          result = false;
        }
      } catch (IOException e) {
        result = false;
      }
    }
    return result;
  }

  /** Compresses PVR input file to create PVRZ output file. */
  public static void compressPvr(Path inFile, Path outFile) throws Exception {
    // handling PVR files
    ByteBuffer bb;
    try (final SeekableByteChannel ch = Files.newByteChannel(inFile, StandardOpenOption.READ)) {
      bb = StreamUtils.getByteBuffer((int) ch.size());
      ch.read(bb);
      bb.position(0);
    }
    if (bb != null) {
      final byte[] buffer = bb.array();
      final byte[] output = Compressor.compress(buffer, 0, buffer.length, true);
      try (final OutputStream os = StreamUtils.getOutputStream(outFile, true)) {
        os.write(output);
      }
    }
  }

  /**
   * Converts the given graphics input file into a PVRZ output file.
   *
   * @param inFile  Input graphics file {@link Path}.
   * @param outFile Output PVRZ file {@link Path}.
   * @param type    PVR compression type.
   * @return A {@link Triple} instance that contains whether the input file is not compatible, number of warnings, and
   *         number of errors.
   */
  public static Triple<Boolean, Integer, Integer> encodePvrz(Path inFile, Path outFile, CompressionType type) {
    final Triple<Boolean, Integer, Integer> retVal = Triple.with(true, 0, 0);

    // handling graphics files
    BufferedImage srcImg = null;
    try {
      srcImg = ColorConvert.toBufferedImage(ImageIO.read(inFile.toFile()), true);
    } catch (Exception e) {
      Logger.trace(e);
    }
    if (srcImg == null) {
      retVal.setValue0(false);
      return retVal;
    }

    // handling "auto" compression format
    int[] pixels = ((DataBufferInt)srcImg.getRaster().getDataBuffer()).getData();
    if (type == CompressionType.AUTO) {
      type = CompressionType.DXT1;
      for (int n = 0; n < pixels.length; n++) {
        final int alpha = pixels[n] >>> 24;
        // alpha in range (32, 224) is considered "true" alpha
        if (alpha > 0x20 && alpha < 0xe0) {
          type = CompressionType.DXT5;
          break;
        }
      }
    }

    // ensure dimensions are always power of two
    int w = nextPowerOfTwo(srcImg.getWidth());
    int h = nextPowerOfTwo(srcImg.getHeight());
    if (w != srcImg.getWidth() || h != srcImg.getHeight()) {
      BufferedImage image = ColorConvert.createCompatibleImage(w, h, true);
      Graphics2D g = image.createGraphics();
      g.drawImage(srcImg, 0, 0, null);
      g.dispose();
      srcImg = image;
      pixels = ((DataBufferInt) srcImg.getRaster().getDataBuffer()).getData();
    }

    // preparing output
    final DxtEncoder.DxtType dxtType;
    final byte[] header;
    if (type == CompressionType.DXT5) {
      dxtType = DxtEncoder.DxtType.DXT5;
      header = createPVRHeader(w, h, 11);
    } else {
      dxtType = DxtEncoder.DxtType.DXT1;
      header = createPVRHeader(w, h, 7);
    }

    // encoding block by block
    final int outSize = DxtEncoder.calcImageSize(w, h, dxtType);
    final byte[] output = new byte[outSize];
    int outOfs = 0;
    int bw = w >> 2;
    int bh = h >> 2;
    final int[] inBlock = new int[16];
    final byte[] outBlock = new byte[DxtEncoder.calcBlockSize(dxtType)];
    for (int y = 0; y < bh; y++) {
      for (int x = 0; x < bw; x++) {
        // starting encoding process
        int ofs = y * w * 4 + x * 4;
        for (int i = 0; i < 4; i++, ofs += w) {
          System.arraycopy(pixels, ofs, inBlock, i * 4, 4);
        }
        try {
          DxtEncoder.encodeBlock(inBlock, outBlock, dxtType);
        } catch (Exception e) {
          retVal.setValue1(retVal.getValue1() + 1);
          Arrays.fill(outBlock, (byte) 0);
        }
        System.arraycopy(outBlock, 0, output, outOfs, outBlock.length);
        outOfs += outBlock.length;
      }
    }

    // finalizing output data
    byte[] pvrz = new byte[header.length + output.length];
    System.arraycopy(header, 0, pvrz, 0, header.length);
    System.arraycopy(output, 0, pvrz, header.length, output.length);
    pvrz = Compressor.compress(pvrz, 0, pvrz.length, true);
    try (OutputStream os = StreamUtils.getOutputStream(outFile, true)) {
      os.write(pvrz);
    } catch (Exception e) {
      retVal.setValue2(retVal.getValue2() + 1);
      Logger.error(e);
    }

    return retVal;
  }

  /**
   * Calculates the first available power of two value that is greater than the specified value.
   *
   * @param value Positive value that should fit into the resulting power of two value.
   * @return A power of two value.
   * @throws IllegalArgumentException if {@code value} is negative or {@code 0}.
   */
  public static int nextPowerOfTwo(int value) {
    if (value <= 0) {
      throw new IllegalArgumentException("Input must be a positive number");
    }

    // using bit manipulation trick
    value--;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;

    return value + 1;
  }

  /**
   * Creates a PVR header based on the parameters specified.
   *
   * @param width       Texture width in pixels.
   * @param height      Texture height in pixels.
   * @param pixelFormat Internal pixel format code.
   * @return The PVR header as byte array.
   */
  public static byte[] createPVRHeader(int width, int height, int pixelFormat) {
    byte[] header = new byte[0x34];
    DynamicArray.putInt(header, 0, 0x03525650); // signature
    DynamicArray.putInt(header, 4, 0); // flags
    DynamicArray.putInt(header, 8, pixelFormat); // pixel format
    DynamicArray.putInt(header, 12, 0); // pixel format (extension)
    DynamicArray.putInt(header, 16, 0); // color space (0=linear rgb)
    DynamicArray.putInt(header, 20, 0); // channel type (0=unsigned byte normalized)
    DynamicArray.putInt(header, 24, height); // height
    DynamicArray.putInt(header, 28, width); // width
    DynamicArray.putInt(header, 32, 1); // depth (in pixels)
    DynamicArray.putInt(header, 36, 1); // # surfaces
    DynamicArray.putInt(header, 40, 1); // # faces
    DynamicArray.putInt(header, 44, 1); // # mipmap levels
    DynamicArray.putInt(header, 48, 0); // # meta data size
    return header;
  }

  // -------------------------- INNER CLASSES --------------------------

}
