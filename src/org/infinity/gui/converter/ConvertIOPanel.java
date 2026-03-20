// Near Infinity - An Infinity Engine Browser and Editor
// Copyright (C) 2001 Jon Olav Hauglid
// See LICENSE.txt for license information

package org.infinity.gui.converter;

import java.awt.Component;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.function.BiFunction;

import javax.swing.JButton;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JTextField;
import javax.swing.event.DocumentEvent;
import javax.swing.event.DocumentListener;
import javax.swing.filechooser.FileNameExtensionFilter;

import org.infinity.gui.ViewerUtil;
import org.infinity.resource.Profile;
import org.infinity.util.io.FileEx;
import org.infinity.util.io.FileManager;
import org.infinity.util.tuples.Triple;

/**
 * A panel that handles single file input and output controls for conversion operations. (Currently used by MOS and TIS
 * conversion dialogs.)
 */
public class ConvertIOPanel extends AbstractConvertPanel implements ActionListener, DocumentListener {
  /** Input file path string has changed. */
  public static final UpdateReason REASON_INPUT_CHANGED = new UpdateReason(1);
  /** Output file path string has changed. */
  public static final UpdateReason REASON_OUTPUT_CHANGED = new UpdateReason(2);

  /**
   * Default output path string generator.
   * <p>
   * It expects input path string and output file extension as parameters and returns a matching output path string.
   * Returns an empty string if an output path string could not be generated.
   * </p>
   * <p>
   * Function parameters are always non-{@code null}.
   * </p>
   */
  public static final BiFunction<String, String, String> DEFAULT_PATH_GENERATOR = (input, ext) -> {
    if (!input.isEmpty()) {
      final Triple<Path, String, String> fileParts = splitPath(input);
      if (fileParts != null) {
        final Path parentPath = (fileParts.getValue0() != null) ? fileParts.getValue0() : Paths.get(".");
        final String fileBase = fileParts.getValue1();
        final boolean isLower = fileBase.codePoints().anyMatch(code -> Character.isLowerCase(code));
        final String fileExt = isLower ? ext.toLowerCase(Locale.ROOT) : ext.toUpperCase(Locale.ROOT);
        final Path outPath = parentPath.resolve(fileBase + '.' + fileExt);
        return outPath.toString();
      }
    }
    return "";
  };

  /**
   * Output path string generator where the filename follows the 8.3 naming scheme to retain compatibility with game
   * resource names.
   * <p>
   * It expects input path string and output file extension as parameters and returns a matching output path string.
   * Returns an empty string if an output path string could not be generated.
   * </p>
   * <p>
   * Function parameters are always non-{@code null}.
   * </p>
   */
  public static final BiFunction<String, String, String> DEFAULT_SHORT_PATH_GENERATOR = (input, ext) -> {
    if (!input.isEmpty()) {
      final Triple<Path, String, String> fileParts = splitPath(input);
      if (fileParts != null) {
        final Path parentPath = (fileParts.getValue0() != null) ? fileParts.getValue0() : Paths.get(".");
        String fileBase = fileParts.getValue1();
        if (fileBase.length() > 8) {
          fileBase = fileBase.substring(0, 8);
        }
        final boolean isLower = fileBase.codePoints().anyMatch(code -> Character.isLowerCase(code));
        final String fileExt = isLower ? ext.toLowerCase(Locale.ROOT) : ext.toUpperCase(Locale.ROOT);
        final Path outPath = parentPath.resolve(fileBase + '.' + fileExt);
        return outPath.toString();
      }
    }
    return "";
  };

  private final List<FileNameExtensionFilter> inputFilterList = new ArrayList<>();
  private final List<FileNameExtensionFilter> outputFilterList = new ArrayList<>();

  private final Component optionsPanel;
  private final String outputExt;
  private final BiFunction<String, String, String> outPathGenerator;

  private JTextField tfInput, tfOutput;
  private JButton bInput, bOutput;
  private Path currentInputDir, currentOutputDir;

  /**
   * A helper method that splits the given path string into a parent {@link Path}, a filename base string and filename
   * extension string.
   *
   * @param path Path string to split.
   * @return {@link Triple} instance with the individual parts of the path if successful, {@code null} otherwise.
   */
  private static Triple<Path, String, String> splitPath(String pathString) {
    if (pathString == null || pathString.isEmpty()) {
      return null;
    }

    final Path path = FileManager.resolve(pathString);
    if (path != null) {
      final Path parentPath = path.getParent();
      final String name = path.getFileName().toString();
      final int pos = name.lastIndexOf('.');
      final String fileBase = (pos >= 0) ? name.substring(0, pos) : name;
      final String fileExt = (pos >= 0) ? name.substring(pos + 1) : "";
      return Triple.with(parentPath, fileBase, fileExt);
    }

    return null;
  }

  /**
   * Initializes a new panel for input and output file definitions.
   *
   * @param title            Title string that is used for the border around the panel.
   * @param initialDir       Initial directory that is used by the open folder dialogs when selecting input or output
   *                           files.
   * @param inputFileFilters Collection of {@link FileNameExtensionFilter} definitions. The first definition will be
   *                           preselected in the input file dialog.
   * @param outputFileFilter {@link FileNameExtensionFilter} definition for the output file.
   * @param outputExtension  File extension for the output file (without leading dot).
   * @param outPathGenerator A function object that generates an output path string from an input path string and the
   *                           output file extension. Specify a custom object or use one of the default generators
   *                           provided by this class.
   * @param optionsPanel     A custom options panel that is added at the bottom section of the panel. Specify
   *                           {@code null} if no further customizations are needed.
   */
  public ConvertIOPanel(String title, Path initialDir, Collection<FileNameExtensionFilter> inputFileFilters,
      FileNameExtensionFilter outputFileFilter, String outputExtension,
      BiFunction<String, String, String> outPathGenerator, Component optionsPanel) {
    super(new GridBagLayout(), title);
    this.outputExt = Objects.requireNonNull(outputExtension);
    this.outPathGenerator = (outPathGenerator != null) ? outPathGenerator : DEFAULT_PATH_GENERATOR;
    this.optionsPanel = optionsPanel;
    init();
    setCurrentInputDirectory(initialDir);
    setCurrentOutputDirectory(initialDir);
    setInputFileFilters(inputFileFilters);
    setOutputFileFilter(outputFileFilter);
  }

  /** Returns the file extension for the output file. */
  public String getOutputExtension() {
    return outputExt;
  }

  /** Returns the current input file path string. */
  public String getInputFile() {
    return tfInput.getText();
  }

  /**
   * Sets a new input file path string. Invalid file paths are not assigned. Optionally autogenerates an output path
   * string from the input path.
   *
   * @param inputFile     File path string that should be assigned to the input path field.
   * @param autoSetOutput Specify {@code true} to automatically generate a matching output filename string and assign it
   *                        to the output path field if it's empty.
   */
  public void setInputFile(String inputFile, boolean autoSetOutput) {
    if (inputFile == null || inputFile.isEmpty()) {
      if (!tfInput.getText().isEmpty()) {
        tfInput.setText("");
      }
      return;
    }

    Path inPath = FileManager.resolve(inputFile);
    if (inPath == null || FileEx.create(inPath).isDirectory()) {
      return;
    }

    final String inPathString = inPath.toString();
    if (!inPathString.equals(tfInput.getText())) {
      tfInput.setText(inPathString);
      currentInputDir = inPath.getParent();

      // auto-filling output path field if empty
      if (autoSetOutput && tfOutput.getText().isEmpty()) {
        final String outPathString = outPathGenerator.apply(inPathString, outputExt);
        if (outPathString != null && !outPathString.isEmpty()) {
          tfOutput.setText(outPathGenerator.apply(inPathString, outputExt));
          currentOutputDir = inPath.getParent();
        }
      }
    }
  }

  /** Returns the current output file path string. */
  public String getOutputFile() {
    return tfOutput.getText();
  }

  /**
   * Sets a new output file path string. Invalid file paths are not assigned.
   *
   * @param outputFile File path string that should be assigned to the output path field.
   */
  public void setOutputFile(String outputFile) {
    if (outputFile == null || outputFile.isEmpty()) {
      if (!tfOutput.getText().isEmpty()) {
        tfOutput.setText("");
      }
      return;
    }

    Path outPath = FileManager.resolve(outputFile);
    if (outPath == null || FileEx.create(outPath).isDirectory()) {
      return;
    }

    final String outPathString = outPath.toString();
    if (!outPathString.equals(tfOutput.getText())) {
      tfOutput.setText(outPathString);
      currentOutputDir = outPath.getParent();
    }
  }

  /**
   * Returns the options panel that was passed to the constructor. Returns {@code null} if no custom options panel was
   * specified.
   */
  public Component getOptionsPanel() {
    return optionsPanel;
  }

  /** Returns the initial input directory used for open file or folder dialogs. */
  public Path getCurrentInputDirectory() {
    return currentInputDir;
  }

  /**
   * Sets a initial input directory that is used for open file or folder dialogs. It is overridden by paths
   * defined in the input directory field.
   *
   * @param dir Directory {@link Path}.
   */
  public void setCurrentInputDirectory(Path dir) {
    if (dir == null || !FileEx.create(dir).isDirectory()) {
      currentInputDir = Profile.getGameRoot();
    } else {
      currentInputDir = dir;
    }
  }

  /** Returns the initial output directory used for open file or folder dialogs. */
  public Path getCurrentOutputDirectory() {
    return currentOutputDir;
  }

  /**
   * Sets a initial output directory that is used for open file or folder dialogs. It is overridden by paths
   * defined in the output directory field.
   *
   * @param dir Directory {@link Path}.
   */
  public void setCurrentOutputDirectory(Path dir) {
    if (dir == null || !FileEx.create(dir).isDirectory()) {
      currentOutputDir = Profile.getGameRoot();
    } else {
      currentOutputDir = dir;
    }
  }

  /** Returns the list of currently defined input file extension filters. */
  public List<FileNameExtensionFilter> getInputFileFilters() {
    return Collections.unmodifiableList(inputFilterList);
  }

  /**
   * Applies the specified list of input file extension filters. Old filters will be discared.
   *
   * @param fileFilters Collection of {@link FileNameExtensionFilter} definitions.
   */
  public void setInputFileFilters(Collection<FileNameExtensionFilter> fileFilters) {
    inputFilterList.clear();
    if (fileFilters != null) {
      for (final FileNameExtensionFilter filter : fileFilters) {
        if (filter != null && !inputFilterList.contains(filter)) {
          inputFilterList.add(filter);
        }
      }
    }
  }

  /** Returns the currently defined output file extension filter. */
  public FileNameExtensionFilter getOutputFileFilter() {
    if (!outputFilterList.isEmpty()) {
      return outputFilterList.get(0);
    }
    return null;
  }

  /**
   * Applies the specified output file extension filter.
   *
   * @param fileFilter {@link FileNameExtensionFilter} definition.
   */
  public void setOutputFileFilter(FileNameExtensionFilter fileFilter) {
    outputFilterList.clear();
    if (fileFilter != null) {
      outputFilterList.add(fileFilter);
    }
  }

  /** Resets the panel to its initial state. */
  public void reset() {
    setInputFile("", false);
    setOutputFile("");
  }

  // --------------------- Begin Interface ActionListener ---------------------

  @Override
  public void actionPerformed(ActionEvent e) {
    if (e.getSource() == bInput) {
      onInput();
    } else if (e.getSource() == bOutput) {
      onOutput();
    }
  }

  // --------------------- End Interface ActionListener ---------------------

  // --------------------- Begin Interface DocumentListener ---------------------

  @Override
  public void insertUpdate(DocumentEvent e) {
    if (e.getDocument() == tfInput.getDocument()) {
      firePanelUpdate(REASON_INPUT_CHANGED);
    } else if (e.getDocument() == tfOutput.getDocument()) {
      firePanelUpdate(REASON_OUTPUT_CHANGED);
    }
  }

  @Override
  public void removeUpdate(DocumentEvent e) {
    if (e.getDocument() == tfInput.getDocument()) {
      firePanelUpdate(REASON_INPUT_CHANGED);
    } else if (e.getDocument() == tfOutput.getDocument()) {
      firePanelUpdate(REASON_OUTPUT_CHANGED);
    }
  }

  @Override
  public void changedUpdate(DocumentEvent e) {
    if (e.getDocument() == tfInput.getDocument()) {
      firePanelUpdate(REASON_INPUT_CHANGED);
    } else if (e.getDocument() == tfOutput.getDocument()) {
      firePanelUpdate(REASON_OUTPUT_CHANGED);
    }
  }

  // --------------------- End Interface DocumentListener ---------------------

  /** Assigns a new input file path string interactively. */
  private void onInput() {
    Path path = null;

    final String text = tfInput.getText();
    if (!text.isEmpty()) {
      path = FileManager.resolve(text);
    }

    if (path == null) {
      path = currentInputDir;
    }

    final Path[] inFile = getOpenFileNames(this, "Choose input graphics file", path, false, inputFilterList, 0);
    if (inFile == null || inFile.length == 0) {
      return;
    }
    currentInputDir = inFile[0].getParent();

    setInputFile(inFile[0].toString(), true);
  }

  /** Assigns a new output file path string interactively. */
  private void onOutput() {
    Path path = null;

    final String text = tfOutput.getText();
    if (!text.isEmpty()) {
      path = FileManager.resolve(text);
    }

    if (path == null) {
      path = currentOutputDir;
    }

    final Path[] outFile = getOpenFileNames(this, "Specify output filename", path, false, outputFilterList, 0);
    if (outFile == null || outFile.length == 0) {
      return;
    }
    currentOutputDir = outFile[0].getParent();

    setOutputFile(outFile[0].toString());
  }

  private void init() {
    final JLabel lInput = new JLabel("Input file:");
    tfInput = new JTextField();
    tfInput.getDocument().addDocumentListener(this);
    bInput = new JButton("...");
    bInput.addActionListener(this);

    final JLabel lOutput = new JLabel("Output file:");
    tfOutput = new JTextField();
    tfOutput.getDocument().addDocumentListener(this);
    bOutput = new JButton("...");
    bOutput.addActionListener(this);

    final GridBagConstraints c = new GridBagConstraints();

    final JPanel panelMain = new JPanel(new GridBagLayout());
    ViewerUtil.setGBC(c, 0, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 0, 0, 0), 0, 0);
    panelMain.add(lInput, c);
    ViewerUtil.setGBC(c, 1, 0, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(0, 8, 0, 0), 0, 0);
    panelMain.add(tfInput, c);
    ViewerUtil.setGBC(c, 2, 0, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(0, 4, 0, 0), 0, 0);
    panelMain.add(bInput, c);

    ViewerUtil.setGBC(c, 0, 1, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(8, 0, 0, 0), 0, 0);
    panelMain.add(lOutput, c);
    ViewerUtil.setGBC(c, 1, 1, 1, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(8, 8, 0, 0), 0, 0);
    panelMain.add(tfOutput, c);
    ViewerUtil.setGBC(c, 2, 1, 1, 1, 0.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.NONE,
        new Insets(8, 4, 0, 0), 0, 0);
    panelMain.add(bOutput, c);

    if (optionsPanel != null) {
      ViewerUtil.setGBC(c, 0, 2, 3, 1, 1.0, 0.0, GridBagConstraints.LINE_START, GridBagConstraints.HORIZONTAL,
          new Insets(8, 0, 0, 0), 0, 0);
      panelMain.add(optionsPanel, c);
    }

    ViewerUtil.setGBC(c, 0, 0, 1, 1, 1.0, 0.0, GridBagConstraints.FIRST_LINE_START, GridBagConstraints.HORIZONTAL,
        new Insets(4, 4, 4, 4), 0, 0);
    add(panelMain, c);
  }
}
