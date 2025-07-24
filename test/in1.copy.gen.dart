// generated file, do not edit

part of 'in1.dart';

// ------------------------------------------------------------
mixin UiInsurancePreviewSimpleDataModelCopyWith {
  String? get title;
  Set get cta;
  List? get image;
  List get image2;

  UiInsurancePreviewSimpleDataModel copyWith({
    NullableValue<String?>? title,
    Set? cta,
    NullableValue<List?>? image,
    List? image2,
  }) => UiInsurancePreviewSimpleDataModel(
    title: title  |  this.title,
    cta: cta  ??  this.cta,
    image: image  |  this.image,
    image2: image2  ??  this.image2,
  );
}
// ------------------------------------------------------------
extension UiInsurancePreviewDataModelCopyWith on UiInsurancePreviewDataModel {
  UiInsurancePreviewDataModel Function({
    String? avatar,
    String tagType,
  }) get copyWith => ({
        Object? avatar = Never,
        Object? tagType = Never,
      }) =>
          UiInsurancePreviewDataModel(
            avatar: avatar._or(this.avatar),
            tagType: tagType._or(this.tagType),
          );
}
// ------------------------------------------------------------

extension on Object? {
  // T? _as<T>() => this is T ? this as T : null;
  T _or<T>(T other) => this is T ? this as T : other;
}
