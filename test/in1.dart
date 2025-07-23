// ignore_for_file: public_member_api_docs, sort_constructors_first

export 'preview.copy.gen.dart';

// +mk:copyWith
class UiInsurancePreviewSimpleDataModel {
  final String title;
  final String cta;
  final String? image;

  const UiInsurancePreviewSimpleDataModel({
    required this.title,
    required this.cta,
    required this.image,
  });
}

// +mk:copyWith
class UiInsurancePreviewDataModel extends UiInsurancePreviewSimpleDataModel {
  final String? avatar;
  final String tagType;

  const UiInsurancePreviewDataModel({
    required this.avatar,
    required super.title,
    required super.cta,
    required super.image,
    required this.tagType,
  });
}
