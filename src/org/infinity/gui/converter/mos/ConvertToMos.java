// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter.mos;

import java.awt.AlphaComposite;
import java.awt.Dimension;
import java.awt.Graphics2D;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Image;
import java.awt.Insets;
import java.awt.Point;
import java.awt.Rectangle;
import java.awt.image.BufferedImage;
import java.awt.image.DataBufferInt;
import java.io.OutputStream;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;

import javax.swing.JOptionPane;
import javax.swing.JPanel;
import javax.swing.filechooser.FileNameExtensionFilter;

import org.infinity.gui.ChildFrame;
import org.infinity.gui.ViewerUtil;
import org.infinity.gui.converter.ConvertButtonsPanel;
import org.infinity.gui.converter.ConvertIOPanel;
import org.infinity.gui.converter.PanelUpdateEvent;
import org.infinity.gui.converter.PanelUpdateListener;
import org.infinity.gui.converter.pvrz.CompressionType;
import org.infinity.gui.converter.pvrz.ConvertToPvrz;
import org.infinity.gui.converter.tis.TileEntry;
import org.infinity.icon.Icons;
import org.infinity.resource.Profile;
import org.infinity.resource.graphics.ColorConvert;
import org.infinity.resource.graphics.Compressor;
import org.infinity.resource.graphics.DxtEncoder;
import org.infinity.util.BinPack2D;
import org.infinity.util.DynamicArray;
import org.infinity.util.IntegerHashMap;
import org.infinity.util.Logger;
import org.infinity.util.io.FileEx;
import org.infinity.util.io.FileManager;
import org.infinity.util.io.StreamUtils;

/**
 * Dialog for converting graphics files into the MOS format (palette-based or pvrz-based).
 */
public class ConvertToMos extends ChildFrame implements PanelUpdateListener {
  private static final FileNameExtensionFilter OUTPUT_FILE_FILTER =
      new FileNameExtensionFilter("MOS files (*.mos)", "mos");

  private static Path currentPath = Profile.getGameRoot();

  private ConvertIOPanel ioPanel;
  private MosOptionsPanel optionsPanel;
  private ConvertButtonsPanel buttonsPanel;

  public ConvertToMos() {
    super("Convert to MOS", true);
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
    currentPath = ioPanel.getCurrentOutputDirectory();
    reset();
    return super.windowClosing(forced);
  }

  // --------------------- End Class ChildFrame ---------------------

  // --------------------- Begin Interface PanelUpdateListener ---------------------

  @Override
  public void panelUpdated(PanelUpdateEvent e) {
    if (e.getSource() == ioPanel) {
      handleIOPanel(e);
    } else if (e.getSource() == buttonsPanel) {
      handleButtonPanel(e);
    }
  }

  // --------------------- End Interface PanelUpdateListener ---------------------

  /** Handles state changes in the IO panel. */
  private void handleIOPanel(PanelUpdateEvent e) {
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

  /** Updates the dialog control state. */
  private void updateStatus() {
    final boolean inputSet = !ioPanel.getInputFile().isEmpty();
    final boolean outputSet = !ioPanel.getOutputFile().isEmpty();

    buttonsPanel.setConvertButtonEnabled(inputSet && outputSet);
  }

  /** Performs the file conversion. */
  private void convert() {
    final boolean isLegacy = optionsPanel.isLegacyVersionSelected();
    final boolean legacyCompressed = optionsPanel.getMosV1Options().isCompressed();
    final int pvrzStartIndex = optionsPanel.getMosV2Options().getPvrzIndex();
    final CompressionType pvrzCompressionType = optionsPanel.getMosV2Options().getCompressionType();
    final boolean closeOnExit = buttonsPanel.isCloseOnExit();

    // preparing input file
    final Path inFile = FileManager.resolve(ioPanel.getInputFile());
    if (!FileEx.create(inFile).isFile()) {
      JOptionPane.showMessageDialog(this, "Input file does not exist:\n" + inFile, "Error", JOptionPane.ERROR_MESSAGE);
      return;
    }

    // preparing output file
    Path outFile = null;
    try {
      outFile = FileManager.resolve(ioPanel.getOutputFile());
    } catch (Exception e) {
      Logger.debug(e);
    }

    if (outFile == null) {
      JOptionPane.showMessageDialog(this, "Invalid output file path specified.", "Error", JOptionPane.ERROR_MESSAGE);
      return;
    }

    final FileEx outFileEx = FileEx.create(outFile);
    if (outFileEx.isDirectory()) {
      JOptionPane.showMessageDialog(this, "Output file cannot be a directory.", "Error", JOptionPane.ERROR_MESSAGE);
      return;
    } else if (outFileEx.exists()) {
      final int result = JOptionPane.showConfirmDialog(this, "MOS output file already exists. Overwrite?",
          "Overwrite", JOptionPane.YES_NO_OPTION, JOptionPane.QUESTION_MESSAGE);
      if (result != JOptionPane.YES_OPTION) {
        return;
      }
    }

    try {
      final MosWorker worker = new MosWorker(this, inFile, outFile, isLegacy, legacyCompressed, pvrzStartIndex, pvrzCompressionType, closeOnExit);
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

  /** Resets session-specific dialog controls. */
  private void reset() {
    ioPanel.reset();
    optionsPanel.reset();
  }

  private void init() {
    setIconImage(Icons.ICON_APPLICATION_16.getIcon().getImage());

    ioPanel = new ConvertIOPanel("Input & Output", currentPath, ColorConvert.GRAPHICS_FILTERS_DEFAULT,
        OUTPUT_FILE_FILTER, "MOS", ConvertIOPanel.DEFAULT_SHORT_PATH_GENERATOR, null);
    ioPanel.addPanelUpdateListener(this);

    optionsPanel = new MosOptionsPanel();

    buttonsPanel = new ConvertButtonsPanel();
    buttonsPanel.addPanelUpdateListener(this);

    final GridBagConstraints c = new GridBagConstraints();

    final JPanel panelMain = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 0, 0, 0), 0, 0);
    panelMain.add(ioPanel, c);
    ViewerUtil.setGBC(c, 0, 1, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelMain.add(optionsPanel, c);
    ViewerUtil.setGBC(c, 0, 2, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 0, 0, 0), 0, 0);
    panelMain.add(buttonsPanel, c);

    setLayout(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 8, 8, 8), 0, 0);
    add(panelMain, c);

    updateStatus();

    pack();
    setMinimumSize(getSize());
    setLocationRelativeTo(getParent());
    setVisible(true);
  }

  /**
   * Converts an image to a palette-based MOS file.
   *
   * @param srcImage    Source {@link Image} to convert.
   * @param outputFile  {@link Path} of the output MOS file.
   * @param compressed  Specifies whether the MOS file should be compressed.
   * @param minProgress Start position for the progress monitor.
   * @param worker      Optional {@link MosWorker} instance for tracking progress.
   * @return {@code true} if the conversion finished successfully, {@code false} if the conversion was cancelled.
   * @throws Exception thrown if an error occurs.
   */
  public static boolean convertV1(BufferedImage srcImage, Path outputFile, boolean compressed, int minProgress,
      MosWorker worker) throws Exception {
    if (srcImage == null) {
      throw new NullPointerException("Source image is null.");
    }
    if (outputFile == null) {
      throw new NullPointerException("Output file is null.");
    }

    // removing alpha-blending
    srcImage = removeBitmapAlpha(srcImage);

    // preparing MOS V1 header
    final int width = srcImage.getWidth();
    final int height = srcImage.getHeight();
    final int cols = (width + 63) / 64;
    final int rows = (height + 63) / 64;
    final int numBlocks = cols * rows;
    final int palOfs = 24;
    final int tableOfs = palOfs + numBlocks * 1024;
    final int dataOfs = tableOfs + numBlocks * 4;
    byte[] dstBuffer = new byte[dataOfs + width * height];
    System.arraycopy("MOS V1  ".getBytes(), 0, dstBuffer, 0, 8);
    DynamicArray.putShort(dstBuffer, 8, (short)width);
    DynamicArray.putShort(dstBuffer, 10, (short)height);
    DynamicArray.putShort(dstBuffer, 12, (short)cols);
    DynamicArray.putShort(dstBuffer, 14, (short)rows);
    DynamicArray.putInt(dstBuffer, 16, 64);
    DynamicArray.putInt(dstBuffer, 20, palOfs);

    // creating list of blocks as int[] arrays
    final List<int[]> blockList = new ArrayList<>(cols * rows);
    for (int y = 0; y < rows; y++) {
      for (int x = 0; x < cols; x++) {
        final int blockX = x * 64;
        final int blockY = y * 64;
        final int blockW = (blockX + 64 < width) ? 64 : (width - blockX);
        final int blockH = (blockY + 64 < height) ? 64 : (height - blockY);
        final int[] rgbArray = new int[blockW * blockH];
        srcImage.getRGB(blockX, blockY, blockW, blockH, rgbArray, 0, blockW);
        blockList.add(rgbArray);
      }
    }

    // applying color reduction to each block
    final int[] palette = new int[255];
    final byte[] blockPalette = new byte[1024];
    final byte[] blockData = new byte[64 * 64];
    final int blockCount = blockList.size();
    final int updateBlock = (blockCount > 300) ? 100 : 10;
    int curPalOfs = palOfs, curTableOfs = tableOfs, curDataOfs = dataOfs;

    final IntegerHashMap<Byte> colorCache = new IntegerHashMap<>(1536); // caching RGBColor -> index
    for (int blockIdx = 0; blockIdx < blockCount; blockIdx++) {
      if (worker != null && worker.isProgressCancelled()) {
        return false;
      }

      if (worker != null && (blockIdx % updateBlock) == 0) {
        worker.setProgressNote("Block " + blockIdx + " / " + blockCount);
        worker.advanceProgressTo(minProgress + blockIdx);
      }

      colorCache.clear();

      final int[] pixels = blockList.get(blockIdx);
      if (ColorConvert.medianCut(pixels, 255, palette, true)) {
        // filling palette
        // first palette entry denotes transparency
        blockPalette[0] = blockPalette[2] = blockPalette[3] = 0;
        blockPalette[1] = (byte)255;
        for (int i = 1; i < 256; i++) {
          blockPalette[(i << 2)]     = (byte)(palette[i - 1] & 0xff);
          blockPalette[(i << 2) + 1] = (byte)((palette[i - 1] >>> 8) & 0xff);
          blockPalette[(i << 2) + 2] = (byte)((palette[i - 1] >>> 16) & 0xff);
          blockPalette[(i << 2) + 3] = 0;
          colorCache.put(palette[i - 1], (byte)(i - 1));
        }
        // filling pixel data
        for (int i = 0; i < pixels.length; i++) {
          if ((pixels[i] & 0xff000000) == 0) {
            blockData[i] = 0;
          } else {
            final Byte palIndex = colorCache.get(pixels[i]);
            if (palIndex != null) {
              blockData[i] = (byte)(palIndex + 1);
            } else {
              byte color = (byte)ColorConvert.getNearestColor(pixels[i], palette, 0.0, null);
              blockData[i] = (byte)(color + 1);
              colorCache.put(pixels[i], color);
            }
          }
        }
      } else {
        // error handling
        throw new Exception("Failed to generate color table for MOS block #" + blockIdx + ". Conversion cancelled.");
      }

      System.arraycopy(blockPalette, 0, dstBuffer, curPalOfs, 1024);
      curPalOfs += 1024;
      DynamicArray.putInt(dstBuffer, curTableOfs, curDataOfs - dataOfs);
      curTableOfs += 4;
      System.arraycopy(blockData, 0, dstBuffer, curDataOfs, pixels.length);
      curDataOfs += pixels.length;
    }

    // optionally compressing to MOSC V1
    if (compressed) {
      dstBuffer = Compressor.compress(dstBuffer, "MOSC", "V1  ");
    }

    // writing TIS file to disk
    try (OutputStream os = StreamUtils.getOutputStream(outputFile, true)) {
      os.write(dstBuffer);
    } catch (Exception e) {
      // error handling
      throw new Exception("Error writing MOS file to disk.", e);
    }

    return true;
  }

  /**
   * Converts an image to a pvrz-based TIS file.
   *
   * @param srcImage        Source {@link Image} to convert.
   * @param outputFile      {@link Path} of the output MOS file. PVRZ files are placed into the same directory as the
   *                          output MOS file.
   * @param pvrzIndex       Start index for PVRZ files.
   * @param compressionType The desired compression type of PVRZ data.
   * @param minProgress     Start position for the progress monitor.
   * @param worker          Optional {@link MosWorker} instance for tracking progress.
   * @return {@code true} if the conversion finished successfully, {@code false} if the conversion was cancelled.
   * @throws Exception thrown if an error occurs.
   */
  public static boolean convertV2(BufferedImage srcImage, Path outputFile, int pvrzIndex,
      CompressionType compressionType, int minProgress, MosWorker worker) throws Exception {
    if (srcImage == null) {
      throw new NullPointerException("Source image is null.");
    }
    if (outputFile == null) {
      throw new NullPointerException("Output file is null.");
    }

    if (pvrzIndex < 0 || pvrzIndex > 99999) {
      throw new IllegalArgumentException("PVRZ index is out of range.");
    }

    final int width = srcImage.getWidth();
    final int height = srcImage.getHeight();
    List<BinPack2D> pageList = new ArrayList<>();
    List<MosEntry> entryList = new ArrayList<>();

    // processing tiles
    generatePageInfo(width, height, pvrzIndex, pageList, entryList);

    // check PVRZ index again
    if (pvrzIndex + pageList.size() > 100000) {
      final String msg = "One or more PVRZ indices exceed the limit.\n"
          + "Please choose a start index smaller than or equal to " + (100000 - pageList.size()) + '.';
      throw new Exception(msg);
    }

    final byte[] dstBuffer = new byte[24 + entryList.size() * 28]; // header + tiles
    int dstOfs = 0;

    // writing MOS header and data
    System.arraycopy("MOS V2  ".getBytes(), 0, dstBuffer, 0, 8);
    DynamicArray.putInt(dstBuffer, 8, width);
    DynamicArray.putInt(dstBuffer, 12, height);
    DynamicArray.putInt(dstBuffer, 16, entryList.size());
    DynamicArray.putInt(dstBuffer, 20, 24);
    dstOfs += 24;
    for (int i = 0; i < entryList.size(); i++, dstOfs += 28) {
      MosEntry entry = entryList.get(i);
      DynamicArray.putInt(dstBuffer, dstOfs, entry.page);
      DynamicArray.putInt(dstBuffer, dstOfs + 4, entry.srcLocation.x);
      DynamicArray.putInt(dstBuffer, dstOfs + 8, entry.srcLocation.y);
      DynamicArray.putInt(dstBuffer, dstOfs + 12, entry.width);
      DynamicArray.putInt(dstBuffer, dstOfs + 16, entry.height);
      DynamicArray.putInt(dstBuffer, dstOfs + 20, entry.dstLocation.x);
      DynamicArray.putInt(dstBuffer, dstOfs + 24, entry.dstLocation.y);
    }

    // writing MOS file to disk
    try (OutputStream os = StreamUtils.getOutputStream(outputFile, true)) {
      os.write(dstBuffer);
    } catch (Exception e) {
      // error handling
      throw new Exception("Error writing MOS file to disk.", e);
    }

    // handling compression format
    DxtEncoder.DxtType dxtType = DxtEncoder.DxtType.DXT1;
    if (compressionType == CompressionType.AUTO) {
      final int[] pixels = ((DataBufferInt)srcImage.getRaster().getDataBuffer()).getData();
      for (int n = 0; n < pixels.length; n++) {
        final int alpha = pixels[n] >>> 24;
        // alpha in range (32, 224) is considered "true" alpha
        if (alpha > 0x20 && alpha < 0xe0) {
          dxtType = DxtEncoder.DxtType.DXT5;
          break;
        }
      }
    } else if (compressionType == CompressionType.DXT5) {
      dxtType = DxtEncoder.DxtType.DXT5;
    }

    // generating PVRZ files
    return createPvrzPages(outputFile, srcImage, pageList, dxtType, entryList, minProgress, worker);
  }

  /** Returns the number of blocks to create in MOS V1 files of the given image dimension. */
  public static int getMosV1BlockCount(int imageWidth, int imageHeight) {
    int retVal = 0;
    if (imageWidth > 0 && imageHeight > 0) {
      final int cols = (imageWidth + 63) / 64;
      final int rows = (imageHeight + 63) / 64;
      retVal = cols * rows;
    }
    return retVal;
  }

  /**
   * Returns a {@code BufferedImage} object without alpha blending.
   *
   * @param image {@link BufferedImage} to process.
   * @return {@link BufferedImage} without alpha blending. Returns the input image if no alpha had to be removed.
   */
  public static BufferedImage removeBitmapAlpha(BufferedImage image) {
    BufferedImage retVal = image;
    if (image != null && image.getType() == BufferedImage.TYPE_INT_ARGB) {
      final int[] srcBuffer = ((DataBufferInt)image.getRaster().getDataBuffer()).getData();
      boolean hasAlpha = false;
      for (int i = srcBuffer.length - 1; i >= 0; i--) {
        final int alpha = srcBuffer[i] >>> 24;
        if (alpha != 0 && alpha != 255) {
          hasAlpha = true;
          break;
        }
      }

      // rendering source image on black background; pure transparency is preserved
      if (hasAlpha) {
        final BufferedImage tmpImage = new BufferedImage(image.getWidth(), image.getHeight(),
            BufferedImage.TYPE_INT_ARGB);
        final int[] dstBuffer = ((DataBufferInt)tmpImage.getRaster().getDataBuffer()).getData();
        for (int i = srcBuffer.length - 1; i >= 0; i--) {
          final int srcAlpha = srcBuffer[i] >>> 24;
          dstBuffer[i] = (srcAlpha == 0) ? 0 : 0xff000000;
        }

        final Graphics2D g = (Graphics2D)tmpImage.getGraphics();
        try {
          g.setComposite(AlphaComposite.SrcOver);
          g.drawImage(image, 0, 0, null);
        } finally {
          g.dispose();
        }
        retVal = tmpImage;
      }
    }
    return retVal;
  }

  /**
   * Generates PVRZ page data that can be used to generate PVRZ files from the given source image.
   *
   * @param imageWidth  Source image width.
   * @param imageHeight Source image height.
   * @param pvrzIndex   Start index for PVRZ files.
   * @param pageList    Empty list where {@link BinPack2D} structures are stored by this method.
   * @param entryList   Empty list where {@link TileEntry} structures are stored by this method.
   * @return Number of generated pvrz page entries.
   * @throws Exception if an error occurs.
   */
  public static int generatePageInfo(int imageWidth, int imageHeight, int pvrzIndex, List<BinPack2D> pageList,
      List<MosEntry> entryList) throws Exception {
    if (imageWidth < 1 || imageHeight < 1) {
      throw new IllegalArgumentException("Image dimension must not be 0 or negative.");
    }

    if (pvrzIndex < 0 || pvrzIndex > 99999) {
      throw new IllegalArgumentException("PVRZ index is out of bounds: " + pvrzIndex);
    }

    if (pageList == null) {
      pageList = new ArrayList<>();
    }

    if (entryList == null) {
      entryList = new ArrayList<>();
    }

    final int pageDim = 1024;
    final BinPack2D.HeuristicRules binPackRule = BinPack2D.HeuristicRules.BOTTOM_LEFT_RULE;

    final int imageSize = imageWidth * imageHeight;
    int x = 0, y = 0, pOfs = 0;
    while (pOfs < imageSize) {
      final int w = Math.min(pageDim, imageWidth - x);
      final int h = Math.min(pageDim, imageHeight - y);
      final Dimension space = new Dimension((w + 3) & ~3, (h + 3) & ~3);
      int pageIdx = -1;
      Rectangle rectMatch = null;
      for (int i = 0, count = pageList.size(); i < count; i++) {
        final BinPack2D packer = pageList.get(i);
        rectMatch = packer.insert(space.width, space.height, binPackRule);
        if (rectMatch.height > 0) {
          pageIdx = i;
          break;
        }
      }

      // create new page?
      if (pageIdx < 0) {
        final BinPack2D packer = new BinPack2D(pageDim, pageDim);
        pageList.add(packer);
        pageIdx = pageList.size() - 1;
        rectMatch = packer.insert(space.width, space.height, binPackRule);
      }

      // register page entry
      final MosEntry entry = new MosEntry(pvrzIndex + pageIdx, new Point(rectMatch.x, rectMatch.y), w, h,
          new Point(x, y));
      entryList.add(entry);

      // advance scanning
      if (x + pageDim >= imageWidth) {
        x = 0;
        y += pageDim;
      } else {
        x += pageDim;
      }
      pOfs = y * imageWidth + x;
    }

    return pageList.size();
  }

  /**
   * Creates PVRZ files according to the specified parameters.
   *
   * @param mosFile     MOS file to create the PVRZ files for.
   * @param srcImage    Source {@link BufferedImage} to convert.
   * @param pageList    List of {@link BinPack2D} structures with page layout information.
   * @param dxtType     Texture compression type.
   * @param entryList   List of MOS block information.
   * @param minProgress Start position for the progress monitor.
   * @param worker      Optional {@link MosWorker} instance for tracking progress.
   * @return {@code true} if the conversion finished successfully, {@code false} if the conversion was cancelled.
   * @throws Exception thrown if an error occurs.
   */
  public static boolean createPvrzPages(Path mosFile, BufferedImage srcImage, List<BinPack2D> pageList,
      DxtEncoder.DxtType dxtType, List<MosEntry> entryList, int minProgress, MosWorker worker) throws Exception {
    final int dxtCode = (dxtType == DxtEncoder.DxtType.DXT5) ? 11 : 7;
    final byte[] output = new byte[DxtEncoder.calcImageSize(1024, 1024, dxtType)];

    int pageMin = Integer.MAX_VALUE;
    int pageMax = -1;
    for (final MosEntry me : entryList) {
      pageMin = Math.min(pageMin, me.page);
      pageMax = Math.max(pageMax, me.page);
    }

    Path pvrzDir = mosFile.getParent();
    if (pvrzDir == null) {
      pvrzDir = Paths.get(".");
    }

    for (int pageIdx = pageMin; pageIdx <= pageMax; pageIdx++) {
      if (worker != null && worker.isProgressCancelled()) {
        return false;
      }

      if (worker != null) {
        final int progress = pageIdx - pageMin;
        final int cur = progress + 1;
        final int max = pageMax - pageMin + 1;
        worker.setProgressNote("PVRZ file " + cur + " / " + max);
        worker.advanceProgressTo(minProgress + progress);
      }

      final Path pvrzFile = pvrzDir.resolve(String.format("MOS%04d.PVRZ", pageIdx));
      final BinPack2D packer = pageList.get(pageIdx - pageMin);
      packer.shrinkBin(true);

      // generating texture image
      final int tw = packer.getBinWidth();
      final int th = packer.getBinHeight();
      final BufferedImage texture = ColorConvert.createCompatibleImage(tw, th, true);
      final Graphics2D g = texture.createGraphics();
      g.setComposite(AlphaComposite.Src);
      g.setColor(ColorConvert.TRANSPARENT_COLOR);
      g.fillRect(0, 0, texture.getWidth(), texture.getHeight());
      for (final MosEntry entry : entryList) {
        if (entry.page == pageIdx) {
          final int sx = entry.dstLocation.x, sy = entry.dstLocation.y;
          final int dx = entry.srcLocation.x, dy = entry.srcLocation.y;
          final int w = entry.width, h = entry.height;
          g.fillRect(dx, dy, w, h);
          g.drawImage(srcImage, dx, dy, dx + w, dy + h, sx, sy, sx + w, sy + h, null);
        }
      }
      g.dispose();

      // compressing PVRZ
      final int[] textureData = ((DataBufferInt)texture.getRaster().getDataBuffer()).getData();
      byte[] pvrz = null;
      try {
        final int outSize = DxtEncoder.calcImageSize(texture.getWidth(), texture.getHeight(), dxtType);
        DxtEncoder.encodeImage(textureData, texture.getWidth(), texture.getHeight(), output, dxtType);
        final byte[] header = ConvertToPvrz.createPVRHeader(texture.getWidth(), texture.getHeight(), dxtCode);
        pvrz = new byte[header.length + outSize];
        System.arraycopy(header, 0, pvrz, 0, header.length);
        System.arraycopy(output, 0, pvrz, header.length, outSize);
        pvrz = Compressor.compress(pvrz, 0, pvrz.length, true);
      } catch (Exception e) {
        throw new Exception("Error while generating PVRZ files:\n" + e.getMessage());
      }

      // writing PVRZ to disk
      try (OutputStream os = StreamUtils.getOutputStream(pvrzFile, true)) {
        os.write(pvrz);
      } catch (Exception e) {
        throw new Exception("Error writing PVRZ file to disk:\n" + pvrzFile, e);
      }
    }

    return true;
  }

  // -------------------------- INNER CLASSES --------------------------

}
